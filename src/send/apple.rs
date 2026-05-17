use std::path::Path;
use std::process::Command;

use crate::error::{Result, RsImsgError};
use crate::types::{SendRequest, SendResult, SendService};

pub fn send(request: &SendRequest) -> Result<SendResult> {
    send_with_db(request, &crate::paths::chat_db_from_env())
}

pub fn send_with_db(request: &SendRequest, db_path: &Path) -> Result<SendResult> {
    let target = resolve_target(request, db_path)?;
    if let Some(file) = &request.file {
        return send_file(&target, file, request.text.as_deref(), request.service);
    }
    let text = request
        .text
        .as_deref()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| RsImsgError::Send("send requires --text and/or --file".into()))?;
    send_text(&target, text, request.service)
}

fn resolve_target(request: &SendRequest, db_path: &Path) -> Result<String> {
    if let Some(to) = &request.to {
        return Ok(to.clone());
    }
    if let Some(guid) = &request.chat_guid {
        return Ok(guid.clone());
    }
    if let Some(id) = &request.chat_identifier {
        return Ok(id.clone());
    }
    if let Some(chat_id) = request.chat_id {
        let store = crate::db::MessageStore::open(db_path)?;
        let chat = store
            .chat_by_id(chat_id)?
            .ok_or_else(|| RsImsgError::Send(format!("chat_id {chat_id} not found")))?;
        return Ok(chat.guid);
    }
    Err(RsImsgError::Send(
        "send requires --to, --chat-id, --chat-guid, or --chat-identifier".into(),
    ))
}

fn service_label(service: SendService) -> &'static str {
    match service {
        SendService::Auto => "auto",
        SendService::Imessage => "iMessage",
        SendService::Sms => "SMS",
    }
}

fn send_text(target: &str, text: &str, service: SendService) -> Result<SendResult> {
    let script = build_send_script(target, text, service_label(service), None)?;
    run_osascript(&script)
}

fn send_file(target: &str, path: &str, caption: Option<&str>, service: SendService) -> Result<SendResult> {
    let file = Path::new(path);
    if !file.is_file() {
        return Err(RsImsgError::Send(format!("file not found: {}", file.display())));
    }
    let staged = stage_attachment(file)?;
    let script = build_send_script(
        target,
        caption.unwrap_or(""),
        service_label(service),
        Some(staged.to_string_lossy().as_ref()),
    )?;
    run_osascript(&script)
}

fn stage_attachment(source: &Path) -> Result<std::path::PathBuf> {
    let dir = crate::paths::default_messages_dir().join("Attachments/rs_imsg");
    std::fs::create_dir_all(&dir)?;
    let name = source
        .file_name()
        .ok_or_else(|| RsImsgError::Send("attachment has no filename".into()))?;
    let dest = dir.join(name);
    std::fs::copy(source, &dest)?;
    Ok(dest)
}

fn build_send_script(target: &str, text: &str, service: &str, file: Option<&str>) -> Result<String> {
    let target_esc = escape_applescript_string(target);
    let text_esc = escape_applescript_string(text);

    let body = if let Some(path) = file {
        let path_esc = escape_applescript_string(path);
        format!(
            r#"
            set theFile to POSIX file "{path_esc}"
            if "{text_esc}" is not "" then
                send "{text_esc}" to targetChat
                delay 0.3
            end if
            send theFile to targetChat
            "#
        )
    } else {
        format!(r#"send "{text_esc}" to targetChat"#)
    };

    let service_type = match service {
        "SMS" => "SMS",
        _ => "iMessage",
    };

    let send_target = if target.contains("chat") && target.contains(';') {
        "targetChat".to_string()
    } else {
        "targetBuddy".to_string()
    };

    let setup = if send_target == "targetChat" {
        format!(
            r#"
            set targetChat to missing value
            set targetHandle to "{target_esc}"
            repeat with c in chats
                if id of c is targetHandle then
                    set targetChat to c
                    exit repeat
                end if
            end repeat
            if targetChat is missing value then
                error "chat not found"
            end if
            "#
        )
    } else {
        format!(
            r#"
            set svc to 1st service whose service type is {service_type}
            set targetBuddy to buddy "{target_esc}" of svc
            "#
        )
    };

    let body = body.replace("targetChat", &send_target);

    Ok(format!(
        r#"
        tell application "Messages"
            {setup}
            {body}
        end tell
        "#
    ))
}

fn escape_applescript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn run_osascript(script: &str) -> Result<SendResult> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| RsImsgError::Send(format!("osascript failed to start: {e}")))?;

    if output.status.success() {
        return Ok(SendResult {
            ok: true,
            detail: None,
        });
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(RsImsgError::Send(format!(
        "osascript exit {}: {stderr}{stdout}",
        output.status
    )))
}
