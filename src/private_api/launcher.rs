use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{Result, RsImessageError};
use crate::private_api::paths::{bridge_ready_lock, resolve_dylib, rpc_inbox, rpc_outbox};
use crate::private_api::sip::require_sip_disabled;

const MESSAGES_BIN: &str = "/System/Applications/Messages.app/Contents/MacOS/Messages";

pub struct Launcher {
    pub dylib_path: PathBuf,
}

impl Launcher {
    pub fn discover() -> Result<Self> {
        let dylib_path = resolve_dylib().ok_or_else(|| {
            RsImessageError::PrivateApi(format!(
                "{} not found; build from openclaw/imsg (make build-dylib) or set RS_IMESSAGE_BRIDGE_DYLIB",
                crate::private_api::protocol::DEFAULT_DYLIB_NAME
            ))
        })?;
        Ok(Self { dylib_path })
    }

    pub fn is_ready(&self) -> bool {
        bridge_ready_lock().is_file()
    }

    pub fn ensure_launched(&self) -> Result<()> {
        if self.is_ready() {
            return Ok(());
        }
        require_sip_disabled()?;
        kill_messages();
        thread::sleep(Duration::from_secs(1));
        let _ = fs::remove_file(bridge_ready_lock());
        ensure_queue_dir(&rpc_inbox())?;
        ensure_queue_dir(&rpc_outbox())?;
        clean_queue_dir(&rpc_inbox())?;
        clean_queue_dir(&rpc_outbox())?;
        launch_with_injection(&self.dylib_path)?;
        wait_for_ready(Duration::from_secs(15))?;
        Ok(())
    }
}

fn ensure_queue_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .map_err(|e| RsImessageError::PrivateApi(format!("mkdir {}: {e}", path.display())))?;
    Ok(())
}

fn clean_queue_dir(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_file() {
            let _ = fs::remove_file(p);
        }
    }
    Ok(())
}

fn kill_messages() {
    let _ = Command::new("/usr/bin/killall").arg("Messages").status();
}

fn launch_with_injection(dylib: &Path) -> Result<()> {
    let dylib = fs::canonicalize(dylib)
        .map_err(|e| RsImessageError::PrivateApi(format!("dylib path: {e}")))?;
    let mut child = Command::new(MESSAGES_BIN);
    child.env("DYLD_INSERT_LIBRARIES", dylib);
    child
        .spawn()
        .map_err(|e| RsImessageError::PrivateApi(format!("launch Messages: {e}")))?;
    Ok(())
}

fn wait_for_ready(timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if bridge_ready_lock().is_file() {
            thread::sleep(Duration::from_millis(500));
            return Ok(());
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err(RsImessageError::PrivateApi(
        "timeout waiting for bridge ready lock".into(),
    ))
}
