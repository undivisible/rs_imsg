//! `rs_imessage` — original agent-first iMessage toolkit for macOS.
//!
//! Design synthesis (not a fork):
//! - **imsg**: JSON lines + JSON-RPC, `watch` with fs events + poll fallback, stderr for humans
//! - **imessage-rs / BlueBubbles**: stable record shapes, group participant lists, attachment metadata
//! - **imessage-kit**: explicit send vs observe semantics, typed chat/message models
//!
//! v0.1 ships read path + AppleScript send + RPC. Optional `private-api` uses the
//! openclaw/imsg Messages dylib (typing, reactions, edit/unsend). FaceTime is a
//! separate [`rs_facetime`](https://github.com/undivisible/rs_facetime) crate.

pub mod client;
pub mod db;
pub mod env;
pub mod error;
#[cfg(feature = "serve")]
pub mod http;
pub mod paths;
pub mod rpc;
pub mod send;
pub mod time;
pub mod types;
pub mod watch;

#[cfg(all(target_os = "macos", feature = "private-api"))]
pub mod private_api;

#[cfg(feature = "serve")]
pub use http::{run as run_bridge, ServeConfig};

pub use client::{Client, ClientConfig};
pub use db::MessageStore;

pub fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unsupported"
    }
}
