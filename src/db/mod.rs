#[cfg(target_os = "macos")]
mod store;
#[cfg(not(target_os = "macos"))]
mod stub_store;

#[cfg(target_os = "macos")]
pub use store::MessageStore;

#[cfg(not(target_os = "macos"))]
pub use stub_store::MessageStore;
