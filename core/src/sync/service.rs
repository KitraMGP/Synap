use std::{collections::HashSet, time::Instant};

use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::{
        note::{Note, NoteSyncRecord},
        tag::{TagReader, TagSyncRecord, TagWriter},
    },
    service::SynapService,
};

use super::protocol::{
    FrameCodec, RecordKey, SyncChannel, SyncConfig, SyncError, SyncMessage, SyncRecord, SyncStats,
    PROTOCOL_VERSION,
};

struct LocalArchive {
    records: Vec<SyncRecord>,
    keys: HashSet<RecordKey>,
}

impl LocalArchive {
    fn new(mut records: Vec<SyncRecord>) -> Self {
        records.sort_by_key(|record| record.key());
        let keys = records.iter().map(SyncRecord::key).collect();
        Self { records, keys }
    }

    fn missing_from(&self, remote_keys: &[RecordKey]) -> Vec<RecordKey> {
        remote_keys
            .iter()
            .filter(|key| !self.keys.contains(*key))
            .cloned()
            .collect()
    }

    fn records_for(&self, keys: &[RecordKey]) -> Vec<SyncRecord> {
        let wanted: HashSet<_> = keys.iter().cloned().collect();
        self.records
            .iter()
            .filter(|record| wanted.contains(&record.key()))
            .cloned()
            .collect()
    }
}

/// Transport-agnostic synchronization service for append-only Synap ledgers.
pub struct SyncService<'a> {
    core: &'a SynapService,
    config: SyncConfig,
}

impl<'a> SyncService<'a> {
    pub fn new(core: &'a SynapService, config: SyncConfig) -> Self {
        Self { core, config }
    }

    pub fn sync_as_initiator<C: SyncChannel>(
        &self,
        channel: &mut C,
    ) -> Result<SyncStats, SyncError> {
        let started = Instant::now();
        let mut stats = SyncStats::default();
        let local = self.collect_local_archive()?;

        self.send_hello(channel, &mut stats)?;
        self.receive_hello(channel, &mut stats)?;

        self.send_manifest(channel, &local, &mut stats)?;
        let remote_manifest = self.receive_manifest(channel, &mut stats)?;

        let need_from_remote = local.missing_from(&remote_manifest);
        self.send_need(channel, &need_from_remote, &mut stats)?;
        let remote_need = self.receive_need(channel, &mut stats)?;

        let outgoing = local.records_for(&remote_need);
        self.send_records(channel, &outgoing, &mut stats)?;
        let incoming = self.receive_records(channel, &mut stats)?;

        let (applied, skipped) = self.apply_remote_records(incoming)?;
        stats.records_applied += applied;
        stats.records_skipped += skipped;

        stats.bytes_sent += FrameCodec::write_message(channel, &SyncMessage::Done)?;
        let (done, bytes) = FrameCodec::read_message(channel)?;
        stats.bytes_received += bytes;
        match done {
            SyncMessage::Done => {}
            other => {
                return Err(SyncError::UnexpectedMessage {
                    expected: "Done",
                    got: other,
                });
            }
        }

        channel.close()?;
        stats.duration_ms = started.elapsed().as_millis() as u64;
        Ok(stats)
    }

    pub fn sync_as_responder<C: SyncChannel>(
        &self,
        channel: &mut C,
    ) -> Result<SyncStats, SyncError> {
        let started = Instant::now();
        let mut stats = SyncStats::default();
        let local = self.collect_local_archive()?;

        self.receive_hello(channel, &mut stats)?;
        self.send_hello(channel, &mut stats)?;

        let remote_manifest = self.receive_manifest(channel, &mut stats)?;
        self.send_manifest(channel, &local, &mut stats)?;

        let need_from_remote = local.missing_from(&remote_manifest);
        let remote_need = self.receive_need(channel, &mut stats)?;
        self.send_need(channel, &need_from_remote, &mut stats)?;

        let incoming = self.receive_records(channel, &mut stats)?;
        let outgoing = local.records_for(&remote_need);
        self.send_records(channel, &outgoing, &mut stats)?;

        let (applied, skipped) = self.apply_remote_records(incoming)?;
        stats.records_applied += applied;
        stats.records_skipped += skipped;

        let (done, bytes) = FrameCodec::read_message(channel)?;
        stats.bytes_received += bytes;
        match done {
            SyncMessage::Done => {}
            other => {
                return Err(SyncError::UnexpectedMessage {
                    expected: "Done",
                    got: other,
                });
            }
        }
        stats.bytes_sent += FrameCodec::write_message(channel, &SyncMessage::Done)?;

        channel.close()?;
        stats.duration_ms = started.elapsed().as_millis() as u64;
        Ok(stats)
    }

    fn collect_local_archive(&self) -> Result<LocalArchive, SyncError> {
        self.core
            .with_read(|tx, reader| {
                let mut records = Vec::new();

                let tag_reader = TagReader::new(tx)?;
                let tags = tag_reader
                    .all()
                    .map_err(redb::Error::from)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(redb::Error::from)?;
                records.extend(
                    tags.into_iter()
                        .map(|tag| SyncRecord::Tag(tag.to_sync_record())),
                );

                let note_ids = reader
                    .note_by_time()
                    .map_err(redb::Error::from)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(redb::Error::from)?;

                let notes = note_ids
                    .iter()
                    .map(|id| {
                        reader
                            .get_by_id(id)?
                            .ok_or_else(|| ServiceError::NotFound(id.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                for note in &notes {
                    records.push(SyncRecord::Note(note.to_sync_record()));

                    let children = reader
                        .children(note)
                        .map_err(redb::Error::from)?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(redb::Error::from)?;
                    records.extend(children.into_iter().map(|child_id| SyncRecord::ReplyLink {
                        parent_id: note.get_id(),
                        child_id,
                    }));

                    let next_versions = reader
                        .next_versions(note)
                        .map_err(redb::Error::from)?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(redb::Error::from)?;
                    records.extend(
                        next_versions
                            .into_iter()
                            .map(|next_id| SyncRecord::EditLink {
                                previous_id: note.get_id(),
                                next_id,
                            }),
                    );
                }

                let tombstones = reader
                    .deleted_note_ids()
                    .map_err(redb::Error::from)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(redb::Error::from)?;
                records.extend(
                    tombstones
                        .into_iter()
                        .map(|note_id| SyncRecord::NoteTombstone { note_id }),
                );

                Ok(LocalArchive::new(records))
            })
            .map_err(Into::into)
    }

    fn apply_remote_records(&self, records: Vec<SyncRecord>) -> Result<(usize, usize), SyncError> {
        let mut unique = HashSet::new();
        let mut tags = Vec::<TagSyncRecord>::new();
        let mut notes = Vec::<NoteSyncRecord>::new();
        let mut reply_links = Vec::<(Uuid, Uuid)>::new();
        let mut edit_links = Vec::<(Uuid, Uuid)>::new();
        let mut tombstones = Vec::<Uuid>::new();
        let mut skipped = 0;

        for record in records {
            if !unique.insert(record.key()) {
                skipped += 1;
                continue;
            }

            match record {
                SyncRecord::Tag(record) => tags.push(record),
                SyncRecord::Note(record) => notes.push(record),
                SyncRecord::ReplyLink {
                    parent_id,
                    child_id,
                } => reply_links.push((parent_id, child_id)),
                SyncRecord::EditLink {
                    previous_id,
                    next_id,
                } => edit_links.push((previous_id, next_id)),
                SyncRecord::NoteTombstone { note_id } => tombstones.push(note_id),
            }
        }

        if unique.is_empty() {
            return Ok((0, skipped));
        }

        self.core.with_write(|tx| {
            let tag_writer = TagWriter::new(tx);

            for record in tags {
                tag_writer.import(record)?;
            }
            for record in notes {
                Note::import(tx, record)?;
            }
            for (parent_id, child_id) in reply_links {
                Note::import_reply_link(tx, &parent_id, &child_id)?;
            }
            for (previous_id, next_id) in edit_links {
                Note::import_edit_link(tx, &previous_id, &next_id)?;
            }
            for note_id in tombstones {
                Note::import_tombstone(tx, &note_id)?;
            }

            Ok(())
        })?;
        self.core.refresh_search_indexes()?;

        Ok((unique.len(), skipped))
    }

    fn send_hello<C: SyncChannel>(
        &self,
        channel: &mut C,
        stats: &mut SyncStats,
    ) -> Result<(), SyncError> {
        stats.bytes_sent += FrameCodec::write_message(
            channel,
            &SyncMessage::Hello {
                version: PROTOCOL_VERSION,
            },
        )?;
        Ok(())
    }

    fn receive_hello<C: SyncChannel>(
        &self,
        channel: &mut C,
        stats: &mut SyncStats,
    ) -> Result<(), SyncError> {
        let (message, bytes) = FrameCodec::read_message(channel)?;
        stats.bytes_received += bytes;

        match message {
            SyncMessage::Hello { version } if version == PROTOCOL_VERSION => Ok(()),
            SyncMessage::Hello { version } => Err(SyncError::ProtocolVersionMismatch {
                local: PROTOCOL_VERSION,
                remote: version,
            }),
            other => Err(SyncError::UnexpectedMessage {
                expected: "Hello",
                got: other,
            }),
        }
    }

    fn send_manifest<C: SyncChannel>(
        &self,
        channel: &mut C,
        local: &LocalArchive,
        stats: &mut SyncStats,
    ) -> Result<(), SyncError> {
        let keys = local.keys.iter().cloned().collect();
        stats.bytes_sent += FrameCodec::write_message(channel, &SyncMessage::Manifest { keys })?;
        Ok(())
    }

    fn receive_manifest<C: SyncChannel>(
        &self,
        channel: &mut C,
        stats: &mut SyncStats,
    ) -> Result<Vec<RecordKey>, SyncError> {
        let (message, bytes) = FrameCodec::read_message(channel)?;
        stats.bytes_received += bytes;

        match message {
            SyncMessage::Manifest { keys } => Ok(keys),
            other => Err(SyncError::UnexpectedMessage {
                expected: "Manifest",
                got: other,
            }),
        }
    }

    fn send_need<C: SyncChannel>(
        &self,
        channel: &mut C,
        keys: &[RecordKey],
        stats: &mut SyncStats,
    ) -> Result<(), SyncError> {
        stats.bytes_sent += FrameCodec::write_message(
            channel,
            &SyncMessage::Need {
                keys: keys.to_vec(),
            },
        )?;
        Ok(())
    }

    fn receive_need<C: SyncChannel>(
        &self,
        channel: &mut C,
        stats: &mut SyncStats,
    ) -> Result<Vec<RecordKey>, SyncError> {
        let (message, bytes) = FrameCodec::read_message(channel)?;
        stats.bytes_received += bytes;

        match message {
            SyncMessage::Need { keys } => Ok(keys),
            other => Err(SyncError::UnexpectedMessage {
                expected: "Need",
                got: other,
            }),
        }
    }

    fn send_records<C: SyncChannel>(
        &self,
        channel: &mut C,
        records: &[SyncRecord],
        stats: &mut SyncStats,
    ) -> Result<(), SyncError> {
        for batch in records.chunks(self.config.max_records_per_message.max(1)) {
            stats.records_sent += batch.len();
            stats.bytes_sent += FrameCodec::write_message(
                channel,
                &SyncMessage::Records {
                    records: batch.to_vec(),
                },
            )?;
        }

        stats.bytes_sent += FrameCodec::write_message(channel, &SyncMessage::RecordsDone)?;
        Ok(())
    }

    fn receive_records<C: SyncChannel>(
        &self,
        channel: &mut C,
        stats: &mut SyncStats,
    ) -> Result<Vec<SyncRecord>, SyncError> {
        let mut records = Vec::new();

        loop {
            let (message, bytes) = FrameCodec::read_message(channel)?;
            stats.bytes_received += bytes;

            match message {
                SyncMessage::Records { records: batch } => {
                    stats.records_received += batch.len();
                    records.extend(batch);
                }
                SyncMessage::RecordsDone => break,
                other => {
                    return Err(SyncError::UnexpectedMessage {
                        expected: "Records or RecordsDone",
                        got: other,
                    });
                }
            }
        }

        Ok(records)
    }
}
