use std::path::Path;

use crate::error::{Result, RsImessageError};
use crate::types::{ChatRecord, MessageRecord};

pub struct MessageStore;

impl MessageStore {
    pub fn open(_path: &Path) -> Result<Self> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    pub fn max_message_rowid(&self) -> Result<i64> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    pub fn list_chats(&self, _limit: usize) -> Result<Vec<ChatRecord>> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    pub fn chat_participants(&self, _chat_id: i64) -> Result<Vec<String>> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    pub fn history(
        &self,
        _chat_id: i64,
        _limit: usize,
        _since_rowid: Option<i64>,
    ) -> Result<Vec<MessageRecord>> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    pub fn messages_after_rowid(
        &self,
        _since_rowid: i64,
        _chat_id: Option<i64>,
        _limit: usize,
    ) -> Result<Vec<MessageRecord>> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    pub fn search(&self, _query: &str, _limit: usize) -> Result<Vec<MessageRecord>> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    pub fn chat_by_id(&self, _chat_id: i64) -> Result<Option<ChatRecord>> {
        Err(RsImessageError::UnsupportedPlatform)
    }
}
