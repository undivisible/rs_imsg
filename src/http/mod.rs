//! Mac-hosted HTTP bridge — run on the machine where Messages.app is signed in.

#[cfg(all(feature = "serve", target_os = "macos"))]
mod server;

#[cfg(all(feature = "serve", target_os = "macos"))]
pub use server::{run, ServeConfig};

#[cfg(all(feature = "serve", not(target_os = "macos")))]
use crate::error::{Result, RsImessageError};

#[cfg(all(feature = "serve", not(target_os = "macos")))]
use crate::client::ClientConfig;
#[cfg(all(feature = "serve", not(target_os = "macos")))]
use crate::watch::WatchOptions;
#[cfg(all(feature = "serve", not(target_os = "macos")))]
use std::net::SocketAddr;

#[cfg(all(feature = "serve", not(target_os = "macos")))]
#[derive(Debug, Clone)]
pub struct ServeConfig {
    pub bind: SocketAddr,
    pub token: String,
    pub client: ClientConfig,
    pub watch: WatchOptions,
}

#[cfg(all(feature = "serve", not(target_os = "macos")))]
pub async fn run(_config: ServeConfig) -> Result<()> {
    Err(RsImessageError::UnsupportedPlatform)
}
