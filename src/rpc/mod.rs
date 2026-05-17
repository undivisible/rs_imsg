use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::MessageStore;
use crate::error::{Result, RsImsgError};
use crate::paths::chat_db_from_env;
use crate::send;
use crate::types::SendRequest;
use crate::watch::{watch_blocking, WatchOptions};

#[derive(Debug, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcErrorObj>,
}

#[derive(Debug, Serialize)]
struct RpcErrorObj {
    code: i32,
    message: String,
}

pub fn run_stdio() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => handle_request(req),
            Err(e) => RpcResponse {
                jsonrpc: "2.0",
                id: Value::Null,
                result: None,
                error: Some(RpcErrorObj {
                    code: -32700,
                    message: format!("parse error: {e}"),
                }),
            },
        };
        serde_json::to_writer(&mut stdout, &response)?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle_request(req: RpcRequest) -> RpcResponse {
    let id = req.id.unwrap_or(Value::Null);
    if req.jsonrpc != "2.0" {
        return err(id, -32600, "invalid jsonrpc version".to_string());
    }
    match dispatch(&req.method, req.params) {
        Ok(result) => RpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        },
        Err(e) => err(id, -32000, e.to_string()),
    }
}

fn err(id: Value, code: i32, message: String) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(RpcErrorObj { code, message }),
    }
}

fn dispatch(method: &str, params: Value) -> Result<Value> {
    let db = chat_db_from_env();
    let store = MessageStore::open(&db)?;
    match method {
        "chats.list" => {
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            Ok(serde_json::to_value(store.list_chats(limit)?)?)
        }
        "messages.history" => {
            let chat_id = params
                .get("chat_id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| RsImsgError::Rpc("messages.history requires chat_id".into()))?;
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            let since = params.get("since_rowid").and_then(|v| v.as_i64());
            Ok(serde_json::to_value(store.history(chat_id, limit, since)?)?)
        }
        "messages.search" => {
            let query = params
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RsImsgError::Rpc("messages.search requires query".into()))?;
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            Ok(serde_json::to_value(store.search(query, limit)?)?)
        }
        "send" => {
            let request: SendRequest = serde_json::from_value(params)?;
            Ok(serde_json::to_value(send::send(&request)?)?)
        }
        "watch.subscribe" => {
            let chat_id = params.get("chat_id").and_then(|v| v.as_i64());
            let since_rowid = params.get("since_rowid").and_then(|v| v.as_i64());
            let poll_ms = params.get("poll_ms").and_then(|v| v.as_u64()).unwrap_or(500);
            let debounce_ms = params.get("debounce_ms").and_then(|v| v.as_u64()).unwrap_or(300);
            watch_blocking(
                &db,
                WatchOptions {
                    chat_id,
                    since_rowid,
                    poll_ms,
                    debounce_ms,
                },
                |event| {
                    let line = serde_json::to_string(&event)?;
                    let mut out = io::stdout().lock();
                    writeln!(out, "{line}")?;
                    out.flush()?;
                    Ok(())
                },
            )?;
            Ok(Value::Null)
        }
        other => Err(RsImsgError::Rpc(format!("unknown method: {other}"))),
    }
}
