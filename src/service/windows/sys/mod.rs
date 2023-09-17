//! Minimal Windows Service API bindings and helpers for our own need.
//!
//! This module only supports managing and running services of type `SERVICE_WIN32_OWN_PROCESS`.
pub use entry::run;
pub use handle::ScHandle;
pub use helper::is_in_windows_service;
pub use manager::ServiceCreateConfig;
pub use manager::ServiceManager;
pub use service::Service;

mod control;
mod entry;
mod handle;
mod helper;
mod manager;
mod service;
