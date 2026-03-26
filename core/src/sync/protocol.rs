use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::models::{note::NoteSyncRecord, tag::TagSyncRecord};

pub const PROTOCOL_VERSION: u8 = 1;
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// A transport-agnostic duplex byte channel for sync.
///
/// Frontends are expected to bridge their own TCP/Bluetooth/WebRTC/etc.
/// implementation to this trait. The core never depends on a real network stack.
pub trait SyncChannel: Read + Write + Send {
    fn close(&mut self) -> io::Result<()> {
        self.flush()
    }
}

impl<T> SyncChannel for T where T: Read + Write + Send {}

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub max_records_per_message: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_records_per_message: 256,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncStats {
    pub records_sent: usize,
    pub records_received: usize,
    pub records_applied: usize,
    pub records_skipped: usize,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordKey {
    Tag(Uuid),
    Note(Uuid),
    ReplyLink { parent_id: Uuid, child_id: Uuid },
    EditLink { previous_id: Uuid, next_id: Uuid },
    NoteTombstone(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncRecord {
    Tag(TagSyncRecord),
    Note(NoteSyncRecord),
    ReplyLink { parent_id: Uuid, child_id: Uuid },
    EditLink { previous_id: Uuid, next_id: Uuid },
    NoteTombstone { note_id: Uuid },
}

impl SyncRecord {
    pub fn key(&self) -> RecordKey {
        match self {
            Self::Tag(record) => RecordKey::Tag(record.id),
            Self::Note(record) => RecordKey::Note(record.id),
            Self::ReplyLink {
                parent_id,
                child_id,
            } => RecordKey::ReplyLink {
                parent_id: *parent_id,
                child_id: *child_id,
            },
            Self::EditLink {
                previous_id,
                next_id,
            } => RecordKey::EditLink {
                previous_id: *previous_id,
                next_id: *next_id,
            },
            Self::NoteTombstone { note_id } => RecordKey::NoteTombstone(*note_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncMessage {
    Hello { version: u8 },
    Manifest { keys: Vec<RecordKey> },
    Need { keys: Vec<RecordKey> },
    Records { records: Vec<SyncRecord> },
    RecordsDone,
    Done,
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("database error: {0}")]
    Db(#[from] redb::Error),

    #[error("transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),

    #[error("commit error: {0}")]
    Commit(#[from] redb::CommitError),

    #[error("service error: {0}")]
    Service(#[from] crate::error::ServiceError),

    #[error("protocol version mismatch: local={local}, remote={remote}")]
    ProtocolVersionMismatch { local: u8, remote: u8 },

    #[error("unexpected message: expected {expected}, got {got:?}")]
    UnexpectedMessage {
        expected: &'static str,
        got: SyncMessage,
    },
}

pub(crate) struct FrameCodec;

impl FrameCodec {
    pub(crate) fn write_message(
        channel: &mut impl SyncChannel,
        message: &SyncMessage,
    ) -> Result<usize, SyncError> {
        let payload = postcard::to_allocvec(message)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        if payload.len() > MAX_FRAME_SIZE {
            return Err(SyncError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "sync frame too large",
            )));
        }

        let len = payload.len() as u32;
        channel.write_all(&len.to_be_bytes())?;
        channel.write_all(&payload)?;

        Ok(payload.len() + 4)
    }

    pub(crate) fn read_message(
        channel: &mut impl SyncChannel,
    ) -> Result<(SyncMessage, usize), SyncError> {
        let mut len_bytes = [0_u8; 4];
        channel.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_FRAME_SIZE {
            return Err(SyncError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "sync frame too large",
            )));
        }

        let mut payload = vec![0_u8; len];
        channel.read_exact(&mut payload)?;
        let message = postcard::from_bytes(&payload)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        Ok((message, len + 4))
    }
}
