use std::path::Path;

use crate::error::{Result, RsImsgError};
use crate::types::WatchEvent;

#[derive(Debug, Clone)]
pub struct WatchOptions {
    pub chat_id: Option<i64>,
    pub since_rowid: Option<i64>,
    pub poll_ms: u64,
    pub debounce_ms: u64,
}

pub fn watch_blocking(
    _db_path: &Path,
    _options: WatchOptions,
    _on_event: impl FnMut(WatchEvent) -> Result<()>,
) -> Result<()> {
    Err(RsImsgError::UnsupportedPlatform)
}
