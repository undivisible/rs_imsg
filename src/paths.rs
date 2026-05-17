use std::path::{Path, PathBuf};

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn default_messages_dir() -> PathBuf {
    home_dir().join("Library/Messages")
}

pub fn default_chat_db() -> PathBuf {
    default_messages_dir().join("chat.db")
}

fn env_path(primary: &str, legacy: &str) -> Option<PathBuf> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(legacy).ok())
        .map(PathBuf::from)
}

pub fn chat_db_from_env() -> PathBuf {
    env_path("RS_IMESSAGE_DB", "RS_IMSG_DB").unwrap_or_else(default_chat_db)
}

pub fn wal_paths(db: &Path) -> [PathBuf; 3] {
    [
        db.to_path_buf(),
        db.with_extension("db-wal"),
        db.with_extension("db-shm"),
    ]
}
