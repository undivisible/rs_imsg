use std::process::Command;

use crate::error::{Result, RsImessageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SipStatus {
    Enabled,
    Disabled,
    Unknown,
}

pub fn current_sip_status() -> SipStatus {
    let output = Command::new("/usr/bin/csrutil").arg("status").output();
    let Ok(output) = output else {
        return SipStatus::Unknown;
    };
    let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
    if text.contains("disabled") {
        SipStatus::Disabled
    } else if text.contains("enabled") {
        SipStatus::Enabled
    } else {
        SipStatus::Unknown
    }
}

pub fn require_sip_disabled() -> Result<()> {
    match current_sip_status() {
        SipStatus::Disabled => Ok(()),
        SipStatus::Enabled => Err(RsImessageError::PrivateApi(
            "System Integrity Protection is enabled; disable SIP in Recovery mode before private-api injection".into(),
        )),
        SipStatus::Unknown => Err(RsImessageError::PrivateApi(
            "could not determine SIP status via csrutil".into(),
        )),
    }
}
