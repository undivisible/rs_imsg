//! `rs_imsg` — original agent-first iMessage toolkit for macOS.
//!
//! Design synthesis (not a fork):
//! - **imsg**: JSON lines + JSON-RPC, `watch` with fs events + poll fallback, stderr for humans
//! - **imessage-rs / BlueBubbles**: stable record shapes, group participant lists, attachment metadata
//! - **imessage-kit**: explicit send vs observe semantics, typed chat/message models
//!
//! v0.1 ships read path + AppleScript send + RPC. Private API (typing, edit, FaceTime) is planned.

pub mod client;
pub mod db;
pub mod error;
pub mod paths;
pub mod rpc;
pub mod send;
pub mod time;
pub mod types;
pub mod watch;

pub use client::{Client, ClientConfig};
pub use db::MessageStore;

pub fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unsupported"
    }
}
