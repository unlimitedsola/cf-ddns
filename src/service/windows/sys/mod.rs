//! Minimal Windows Service API bindings and helpers for our own need.
//!
//! This module only supports managing and running services of type `SERVICE_WIN32_OWN_PROCESS`.
pub use sc_handle::ScHandle;
pub use service::Service;
pub use service_helper::is_in_windows_service;
pub use service_helper::parse_service_entry_arguments;
pub use service_manager::ServiceCreateConfig;
pub use service_manager::ServiceManager;

mod sc_handle;
mod service;
pub mod service_control;
mod service_helper;
pub mod service_main;
mod service_manager;
