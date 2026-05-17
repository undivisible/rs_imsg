use std::path::Path;

use crate::error::{Result, RsImsgError};
use crate::types::{SendRequest, SendResult};

pub fn send(request: &SendRequest) -> Result<SendResult> {
    send_with_db(request, Path::new(""))
}

pub fn send_with_db(_request: &SendRequest, _db_path: &Path) -> Result<SendResult> {
    Err(RsImsgError::UnsupportedPlatform)
}
