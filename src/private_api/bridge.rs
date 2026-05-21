use serde_json::{json, Value};

use crate::error::Result;
use std::path::PathBuf;

use crate::private_api::ipc::{invoke_default, BridgeResponse};
use crate::private_api::launcher::Launcher;
use crate::private_api::paths::events_log;
use crate::private_api::protocol::BridgeAction;

pub struct BridgeClient {
    _launcher: Launcher,
}

impl BridgeClient {
    pub fn connect() -> Result<Self> {
        let launcher = Launcher::discover()?;
        launcher.ensure_launched()?;
        Ok(Self {
            _launcher: launcher,
        })
    }

    pub fn is_ready() -> bool {
        Launcher::discover().map(|l| l.is_ready()).unwrap_or(false)
    }

    pub fn events_log_path() -> PathBuf {
        events_log()
    }

    pub fn ping(&self) -> Result<BridgeResponse> {
        self.invoke(BridgeAction::Ping, json!({}))
    }

    pub fn start_typing(&self, chat_guid: &str) -> Result<BridgeResponse> {
        self.invoke(BridgeAction::StartTyping, json!({ "chatGuid": chat_guid }))
    }

    pub fn stop_typing(&self, chat_guid: &str) -> Result<BridgeResponse> {
        self.invoke(BridgeAction::StopTyping, json!({ "chatGuid": chat_guid }))
    }

    pub fn send_message(
        &self,
        chat_guid: &str,
        text: &str,
        temp_guid: Option<&str>,
    ) -> Result<BridgeResponse> {
        let mut params = json!({
            "chatGuid": chat_guid,
            "message": text,
        });
        if let Some(g) = temp_guid {
            params["tempGuid"] = json!(g);
        }
        self.invoke(BridgeAction::SendMessage, params)
    }

    pub fn send_reaction(
        &self,
        chat_guid: &str,
        message_guid: &str,
        reaction: &str,
    ) -> Result<BridgeResponse> {
        self.invoke(
            BridgeAction::SendReaction,
            json!({
                "chatGuid": chat_guid,
                "selectedMessageGuid": message_guid,
                "reaction": reaction,
            }),
        )
    }

    pub fn edit_message(
        &self,
        chat_guid: &str,
        message_guid: &str,
        new_text: &str,
    ) -> Result<BridgeResponse> {
        self.invoke(
            BridgeAction::EditMessage,
            json!({
                "chatGuid": chat_guid,
                "selectedMessageGuid": message_guid,
                "editedMessage": new_text,
            }),
        )
    }

    pub fn unsend_message(&self, chat_guid: &str, message_guid: &str) -> Result<BridgeResponse> {
        self.invoke(
            BridgeAction::UnsendMessage,
            json!({
                "chatGuid": chat_guid,
                "selectedMessageGuid": message_guid,
            }),
        )
    }

    pub fn mark_chat_read(&self, chat_guid: &str) -> Result<BridgeResponse> {
        self.invoke(BridgeAction::MarkChatRead, json!({ "chatGuid": chat_guid }))
    }

    pub fn create_chat(&self, addresses: &[&str]) -> Result<BridgeResponse> {
        self.invoke(BridgeAction::CreateChat, json!({ "addresses": addresses }))
    }

    pub fn invoke(&self, action: BridgeAction, params: Value) -> Result<BridgeResponse> {
        invoke_default(action, params)
    }
}
