use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::db::MessageStore;
use crate::error::{Result, RsImsgError};
use crate::paths::wal_paths;
use crate::types::{WatchEvent, WatchEventKind};

#[derive(Debug, Clone)]
pub struct WatchOptions {
    pub chat_id: Option<i64>,
    pub since_rowid: Option<i64>,
    pub poll_ms: u64,
    pub debounce_ms: u64,
}

pub fn watch_blocking(
    db_path: &Path,
    options: WatchOptions,
    mut on_event: impl FnMut(WatchEvent) -> Result<()>,
) -> Result<()> {
    let store = MessageStore::open(db_path)?;
    let mut cursor = options
        .since_rowid
        .unwrap_or_else(|| store.max_message_rowid().unwrap_or(0));

    let (notify_tx, notify_rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = notify_tx.send(res);
        },
        notify::Config::default(),
    )
    .map_err(|e| RsImsgError::Watch(e.to_string()))?;

    for path in wal_paths(db_path) {
        if path.exists() {
            let _ = watcher.watch(&path, RecursiveMode::NonRecursive);
        }
    }

    let debounce = Duration::from_millis(options.debounce_ms.max(50));
    let poll_every = Duration::from_millis(options.poll_ms.max(200));

    loop {
        let _ = notify_rx.recv_timeout(poll_every);
        std::thread::sleep(debounce);

        let store = MessageStore::open(db_path)?;
        let batch = store.messages_after_rowid(cursor, options.chat_id, 200)?;
        for message in batch {
            cursor = message.id;
            on_event(WatchEvent {
                kind: WatchEventKind::NewMessage,
                message,
            })?;
        }
    }
}
