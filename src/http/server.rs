use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::client::{Client, ClientConfig};
use crate::error::{Result as ImsgResult, RsImsgError};
use crate::types::{ChatRecord, MessageRecord, SendRequest, SendResult, WatchEvent};

type HttpResult<T> = std::result::Result<T, StatusCode>;
use crate::watch::WatchOptions;

#[derive(Debug, Clone)]
pub struct ServeConfig {
    pub bind: SocketAddr,
    pub token: String,
    pub client: ClientConfig,
    pub watch: WatchOptions,
}

#[derive(Clone)]
struct AppState {
    client: Client,
    token: String,
    events: broadcast::Sender<WatchEvent>,
}

#[derive(Debug, Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatsQuery {
    limit: Option<usize>,
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HistoryBody {
    chat_id: i64,
    limit: Option<usize>,
    since_rowid: Option<i64>,
}

#[derive(Debug, Serialize)]
struct Envelope<T> {
    status: u16,
    message: &'static str,
    data: T,
}

fn ok<T: Serialize>(data: T) -> Json<Envelope<T>> {
    Json(Envelope {
        status: 200,
        message: "Success",
        data,
    })
}

fn authorize(headers: &HeaderMap, query_token: Option<&str>, expected: &str) -> bool {
    let from_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);
    let token = from_header.or(query_token).unwrap_or("");
    !token.is_empty() && token == expected
}

async fn health() -> &'static str {
    "ok"
}

async fn ping() -> Json<Envelope<&'static str>> {
    ok("pong")
}

async fn list_chats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ChatsQuery>,
) -> HttpResult<Json<Envelope<Vec<ChatRecord>>>> {
    if !authorize(&headers, q.token.as_deref(), &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let limit = q.limit.unwrap_or(20);
    let chats = state
        .client
        .list_chats(limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(ok(chats))
}

async fn message_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<TokenQuery>,
    Json(body): Json<HistoryBody>,
) -> HttpResult<Json<Envelope<Vec<MessageRecord>>>> {
    if !authorize(&headers, q.token.as_deref(), &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let limit = body.limit.unwrap_or(50);
    let messages = state
        .client
        .history(body.chat_id, limit, body.since_rowid)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(ok(messages))
}

async fn message_send(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<TokenQuery>,
    Json(body): Json<SendRequest>,
) -> HttpResult<Json<Envelope<SendResult>>> {
    if !authorize(&headers, q.token.as_deref(), &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let result = state
        .client
        .send(&body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(ok(result))
}

async fn events_sse(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<TokenQuery>,
) -> HttpResult<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    if !authorize(&headers, q.token.as_deref(), &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let rx = state.events.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|item| {
        let ev = item.ok()?;
        let json = serde_json::to_string(&ev).ok()?;
        Some(Ok(Event::default().data(json)))
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub async fn run(config: ServeConfig) -> ImsgResult<()> {
    let client = Client::open(config.client)?;
    let (events_tx, _) = broadcast::channel::<WatchEvent>(512);

    let watch_client = client.clone();
    let watch_opts = config.watch.clone();
    let watch_tx = events_tx.clone();
    tokio::task::spawn_blocking(move || {
        let db_path = watch_client.db_path().to_path_buf();
        let _ = crate::watch::watch_blocking(&db_path, watch_opts, |ev| {
            let _ = watch_tx.send(ev);
            Ok(())
        });
    });

    let state = Arc::new(AppState {
        client,
        token: config.token,
        events: events_tx,
    });

    let api = Router::new()
        .route("/health", get(health))
        .route("/api/v1/ping", get(ping))
        .route("/api/v1/chats", get(list_chats))
        .route("/api/v1/messages/history", post(message_history))
        .route("/api/v1/messages/send", post(message_send))
        .route("/api/v1/events", get(events_sse))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .map_err(|e| RsImsgError::Other(e.to_string()))?;
    eprintln!("rs_imsg bridge listening on http://{}", config.bind);
    axum::serve(listener, api)
        .await
        .map_err(|e| RsImsgError::Other(e.to_string()))?;
    Ok(())
}
