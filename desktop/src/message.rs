//! Message definitions for the desktop application.

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    SearchQueryChanged(String),
    RunSearch,
    ClearSearch,
    ComposeContentChanged(String),
    ComposeTagsChanged(String),
    CreateNote,
    ReplyToSelected,
    SelectNote(String),
    DetailContentChanged(String),
    DetailTagsChanged(String),
    SaveNewVersion,
    DeleteSelected,
    RestoreNote(String),
    SeedDemo,
}
