use crate::{error::NoteError, models::note::NoteReader, views::note_view::NoteView};

pub struct TimelineView<'a> {
    reader: &'a NoteReader<'a>,
}

impl<'a> TimelineView<'a> {
    pub fn new(reader: &'a NoteReader<'a>) -> Self {
        Self { reader }
    }

    /// 获取全局的最新动态 (按时间倒序)
    pub fn recent(
        &self,
    ) -> Result<impl Iterator<Item = Result<NoteView<'_, 'a>, NoteError>> + '_, redb::Error> {
        let raw_iter = self.reader.note_by_time()?.rev();

        let views = raw_iter.filter_map(|res| match res {
            Ok(uuid) => match self.reader.get_by_id(&uuid) {
                Ok(Some(note)) if !note.is_deleted() => Some(Ok(NoteView::new(self.reader, note))),
                Ok(Some(_)) => None, // 过滤已删除
                Ok(None) => Some(Err(NoteError::IdNotFound { id: uuid })),
                Err(e) => Some(Err(NoteError::Db(e.into()))),
            },
            Err(e) => Some(Err(NoteError::Db(e.into()))),
        });

        Ok(views)
    }
}
