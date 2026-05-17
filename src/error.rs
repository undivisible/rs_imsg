use thiserror::Error;

#[derive(Debug, Error)]
pub enum RsImessageError {
    #[error("rs_imessage requires macOS (Messages.app + chat.db)")]
    UnsupportedPlatform,

    #[cfg(target_os = "macos")]
    #[error("database: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("watch: {0}")]
    Watch(String),

    #[error("send: {0}")]
    Send(String),

    #[error("rpc: {0}")]
    Rpc(String),

    #[error("private-api: {0}")]
    PrivateApi(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RsImessageError>;
