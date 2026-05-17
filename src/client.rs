use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::thread;

use tokio::sync::mpsc;

use crate::db::MessageStore;
use crate::error::{Result, RsImessageError};
use crate::paths::chat_db_from_env;
use crate::send;
use crate::types::{ChatRecord, MessageRecord, SendRequest, SendResult, WatchEvent};
#[cfg(target_os = "macos")]
use crate::watch::watch_blocking;
use crate::watch::WatchOptions;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub chat_db_path: PathBuf,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            chat_db_path: chat_db_from_env(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    db_path: PathBuf,
}

impl Client {
    pub fn open(config: ClientConfig) -> Result<Self> {
        MessageStore::open(&config.chat_db_path)?;
        Ok(Self {
            db_path: config.chat_db_path,
        })
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn list_chats(&self, limit: usize) -> Result<Vec<ChatRecord>> {
        MessageStore::open(&self.db_path)?.list_chats(limit)
    }

    pub fn history(
        &self,
        chat_id: i64,
        limit: usize,
        since_rowid: Option<i64>,
    ) -> Result<Vec<MessageRecord>> {
        MessageStore::open(&self.db_path)?.history(chat_id, limit, since_rowid)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<MessageRecord>> {
        MessageStore::open(&self.db_path)?.search(query, limit)
    }

    pub fn send(&self, request: &SendRequest) -> Result<SendResult> {
        send::send_with_db(request, &self.db_path)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn watch(
        &self,
        _options: WatchOptions,
    ) -> Result<mpsc::Receiver<Result<WatchEvent>>> {
        Err(RsImessageError::UnsupportedPlatform)
    }

    #[cfg(target_os = "macos")]
    pub fn watch(
        &self,
        options: WatchOptions,
    ) -> Result<mpsc::Receiver<Result<WatchEvent>>> {
        let db_path = self.db_path.clone();
        MessageStore::open(&db_path)?;

        let (tx, rx) = mpsc::channel(256);
        thread::spawn(move || {
            let send_res = watch_blocking(&db_path, options, |ev| {
                if tx.blocking_send(Ok(ev)).is_err() {
                    return Err(RsImessageError::Watch("watch receiver dropped".into()));
                }
                Ok(())
            });
            if let Err(e) = send_res {
                let _ = tx.blocking_send(Err(e));
            }
        });

        Ok(rx)
    }

    #[cfg(all(target_os = "macos", feature = "private-api"))]
    pub fn bridge(&self) -> Result<crate::private_api::BridgeClient> {
        let _ = self;
        crate::private_api::BridgeClient::connect()
    }
}
