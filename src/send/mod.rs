#[cfg(target_os = "macos")]
mod apple;
#[cfg(not(target_os = "macos"))]
mod stub;

#[cfg(target_os = "macos")]
pub use apple::{send, send_with_db};

#[cfg(not(target_os = "macos"))]
pub use stub::{send, send_with_db};
