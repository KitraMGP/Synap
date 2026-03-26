use uuid::Uuid;

use crate::{
    dto::NoteDTO,
    error::NoteError,
    models::{
        note::{Note, NoteReader},
        tag::TagReader,
    },
};

/// NoteView：零成本的业务视图包装
/// 'a = NoteReader 的借用生命周期
/// 'b = NoteReader 内部 transaction 的生命周期
pub struct NoteView<'a, 'b: 'a> {
    reader: &'a NoteReader<'b>,
    pub note: Note,
}

impl<'a, 'b> NoteView<'a, 'b> {
    /// 构造一个视图导航器（零成本，只是借用 Reader）
    pub fn new(reader: &'a NoteReader<'b>, note: Note) -> Self {
        Self { reader, note }
    }

    /// 从 ID 构造 View
    pub fn from_id(reader: &'a NoteReader<'b>, id: Uuid) -> Result<Self, NoteError> {
        let note = reader
            .get_by_id(&id)
            .map_err(|err| NoteError::Db(err))?
            .ok_or(NoteError::IdNotFound { id })?;
        Ok(Self::new(reader, note))
    }

    pub fn from_short_id(reader: &'a NoteReader<'b>, id: [u8; 8]) -> Result<Self, NoteError> {
        let note = reader
            .get_by_short_id(&id)
            .map_err(|err| NoteError::Db(err))?
            .ok_or(NoteError::ShortIdNotFound { id })?;
        Ok(Self::new(reader, note))
    }

    /// UUID 转换为 NoteView（内部方法）
    fn uuid_to_view(
        &'a self,
        iter: impl Iterator<Item = Result<Uuid, NoteError>> + 'a,
    ) -> impl Iterator<Item = Result<NoteView<'a, 'b>, NoteError>> + 'a {
        iter.filter_map(move |uuid_res| match uuid_res {
            Ok(id) => match self.reader.get_by_id(&id) {
                Ok(Some(note)) if !note.is_deleted() => Some(Ok(NoteView::new(self.reader, note))),
                Ok(Some(_)) => None, // 过滤已删除
                Ok(None) => Some(Err(NoteError::IdNotFound { id })),
                Err(e) => Some(Err(NoteError::Db(e.into()))),
            },
            Err(e) => Some(Err(e)),
        })
    }

    /// 向上溯源：返回父节点的迭代器
    pub fn parents(
        &'a self,
    ) -> Result<impl Iterator<Item = Result<NoteView<'a, 'b>, NoteError>> + 'a, NoteError> {
        let iter = self
            .reader
            .parents(&self.note)
            .map_err(|e| NoteError::Db(e.into()))?;
        let aligned = iter.map(|res| res.map_err(|e| NoteError::Db(e.into())));
        Ok(self.uuid_to_view(aligned))
    }

    /// 向下推演：返回子节点的迭代器（过滤已删除）
    pub fn children(
        &'a self,
    ) -> Result<impl Iterator<Item = Result<NoteView<'a, 'b>, NoteError>> + 'a, NoteError> {
        let iter = self
            .reader
            .children(&self.note)
            .map_err(|e| NoteError::Db(e.into()))?;
        let aligned = iter.map(|res| res.map_err(|e| NoteError::Db(e.into())));
        Ok(self.uuid_to_view(aligned))
    }

    /// 获取历史版本沿革（过滤已删除）
    pub fn history(
        &'a self,
    ) -> Result<impl Iterator<Item = Result<NoteView<'a, 'b>, NoteError>> + 'a, NoteError> {
        let iter = self
            .reader
            .previous_versions(&self.note)
            .map_err(|e| NoteError::Db(e.into()))?;
        let aligned = iter.map(|res| res.map_err(|e| NoteError::Db(e.into())));
        Ok(self.uuid_to_view(aligned))
    }

    /// 获取下一个版本（过滤已删除）
    pub fn next_version(
        &'a self,
    ) -> Result<impl Iterator<Item = Result<NoteView<'a, 'b>, NoteError>> + 'a, NoteError> {
        let iter = self
            .reader
            .next_versions(&self.note)
            .map_err(|e| NoteError::Db(e.into()))?;
        let aligned = iter.map(|res| res.map_err(|e| NoteError::Db(e.into())));
        Ok(self.uuid_to_view(aligned))
    }

    /// 获取当前版本在编辑链上的其他版本（过滤已删除）
    pub fn other_versions(
        &'a self,
    ) -> Result<impl Iterator<Item = Result<NoteView<'a, 'b>, NoteError>> + 'a, NoteError> {
        let iter = self
            .reader
            .other_versions(&self.note)
            .map_err(|e| NoteError::Db(e.into()))?;
        let aligned = iter.map(|res| res.map_err(|e| NoteError::Db(e.into())));
        Ok(self.uuid_to_view(aligned))
    }

    /// 获取当前节点的标签
    pub fn tags(&self) -> Result<Vec<crate::models::tag::Tag>, NoteError> {
        let tag_reader = TagReader::new(self.reader.tx()).map_err(|e| NoteError::Db(e))?;
        self.note
            .tags()
            .iter()
            .map(|id| {
                tag_reader
                    .get_by_id(id)
                    .map_err(|err| NoteError::Db(err.into()))?
                    .ok_or_else(|| NoteError::IdNotFound { id: *id })
            })
            .collect()
    }

    /// 组装 DTO
    pub fn to_dto(&self) -> Result<NoteDTO, NoteError> {
        let tags: Vec<String> = self
            .tags()?
            .iter()
            .map(|t| t.get_content().to_string())
            .collect();
        let (seconds, nanos) = self
            .note
            .get_id()
            .get_timestamp()
            .ok_or(NoteError::IdNotFound {
                id: self.note.get_id(),
            })?
            .to_unix();

        Ok(NoteDTO {
            id: self.note.get_id().to_string(),
            content: self.note.content().to_string(),
            tags,
            created_at: seconds.saturating_mul(1000) + u64::from(nanos / 1_000_000),
        })
    }

    pub fn get_note(&self) -> &Note {
        &self.note
    }
}
