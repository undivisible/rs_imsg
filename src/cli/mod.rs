use std::io::{self, Write};

use clap::{Parser, Subcommand};
use serde::Serialize;

use rs_imsg::db::MessageStore;
use rs_imsg::error::Result;
use rs_imsg::paths::chat_db_from_env;
use rs_imsg::send;
use rs_imsg::types::SendRequest;
use rs_imsg::watch::{watch_blocking, WatchOptions};

#[derive(Parser, Debug)]
#[command(name = "rs_imsg", about = "Agent-first iMessage toolkit for macOS")]
pub struct Cli {
    #[arg(long, global = true, help = "Path to chat.db (default: ~/Library/Messages/chat.db)")]
    pub db: Option<std::path::PathBuf>,

    #[arg(long, global = true, help = "Emit one JSON object per line on stdout")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Chats {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Group {
        #[arg(long)]
        chat_id: i64,
    },
    History {
        #[arg(long)]
        chat_id: i64,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        since_rowid: Option<i64>,
    },
    Watch {
        #[arg(long)]
        chat_id: Option<i64>,
        #[arg(long)]
        since_rowid: Option<i64>,
        #[arg(long, default_value_t = 500)]
        poll_ms: u64,
        #[arg(long, default_value_t = 300)]
        debounce_ms: u64,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Send {
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        chat_id: Option<i64>,
        #[arg(long)]
        chat_guid: Option<String>,
        #[arg(long)]
        chat_identifier: Option<String>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    Rpc,
}

pub fn run(cli: Cli) -> Result<()> {
  match cli.command {
        Commands::Rpc => rs_imsg::rpc::run_stdio(),
        other => run_command(cli.db.as_deref(), cli.json, other),
    }
}

fn run_command(db: Option<&std::path::Path>, json: bool, command: Commands) -> Result<()> {
    let path = db.map(std::path::PathBuf::from).unwrap_or_else(chat_db_from_env);
    match command {
        Commands::Chats { limit } => {
            let store = MessageStore::open(&path)?;
            emit(json, &store.list_chats(limit)?)?;
        }
        Commands::Group { chat_id } => {
            let store = MessageStore::open(&path)?;
            let chat = store
                .chat_by_id(chat_id)?
                .ok_or_else(|| rs_imsg::error::RsImsgError::Other(format!("chat {chat_id} not found")))?;
            emit(json, &chat)?;
        }
        Commands::History {
            chat_id,
            limit,
            since_rowid,
        } => {
            let store = MessageStore::open(&path)?;
            for msg in store.history(chat_id, limit, since_rowid)? {
                emit(json, &msg)?;
            }
        }
        Commands::Watch {
            chat_id,
            since_rowid,
            poll_ms,
            debounce_ms,
        } => {
            watch_blocking(
                &path,
                WatchOptions {
                    chat_id,
                    since_rowid,
                    poll_ms,
                    debounce_ms,
                },
                |event| {
                    emit(json, &event)?;
                    Ok(())
                },
            )?;
        }
        Commands::Search { query, limit } => {
            let store = MessageStore::open(&path)?;
            for msg in store.search(&query, limit)? {
                emit(json, &msg)?;
            }
        }
        Commands::Send {
            to,
            chat_id,
            chat_guid,
            chat_identifier,
            text,
            file,
        } => {
            let request = SendRequest {
                to,
                chat_id,
                chat_guid,
                chat_identifier,
                text,
                file,
                service: Default::default(),
            };
            let result = send::send(&request)?;
            emit(json, &result)?;
        }
        Commands::Rpc => unreachable!(),
    }
    Ok(())
}

fn emit<T: Serialize>(json: bool, value: &T) -> Result<()> {
    if json {
        serde_json::to_writer(io::stdout(), value)?;
        writeln!(io::stdout())?;
    } else {
        writeln!(io::stdout(), "{}", serde_json::to_string_pretty(value)?)?;
    }
    Ok(())
}
