use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRecord {
    pub id: i64,
    pub name: Option<String>,
    pub identifier: String,
    pub guid: String,
    pub service: Option<String>,
    pub is_group: bool,
    pub participants: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: i64,
    pub guid: String,
    pub chat_id: i64,
    pub chat_identifier: String,
    pub chat_guid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_name: Option<String>,
    pub participants: Vec<String>,
    pub is_group: bool,
    pub sender: Option<String>,
    pub is_from_me: bool,
    pub text: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_guid: Option<String>,
    pub attachments: Vec<AttachmentMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub filename: Option<String>,
    pub mime_type: Option<String>,
    pub byte_count: Option<i64>,
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WatchEvent {
    pub kind: WatchEventKind,
    pub message: MessageRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchEventKind {
    NewMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendRequest {
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub chat_id: Option<i64>,
    #[serde(default)]
    pub chat_guid: Option<String>,
    #[serde(default)]
    pub chat_identifier: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub service: SendService,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SendService {
    #[default]
    Auto,
    Imessage,
    Sms,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
