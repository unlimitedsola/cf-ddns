use anyhow::Result;
use windows::Win32::System::Services;

pub use sc_handle::ScHandle;
pub use service::Service;
pub use service_config::ServiceConfig;
pub use service_manager::ServiceManager;

mod sc_handle;
mod service;
mod service_config;
mod service_manager;
