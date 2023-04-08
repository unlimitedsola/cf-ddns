pub use sc_handle::ScHandle;
pub use service::Service;
pub use service_config::ServiceConfig;
pub use service_helper::is_in_windows_service;
pub use service_manager::ServiceManager;

mod sc_handle;
mod service;
mod service_config;
pub mod service_control;
mod service_helper;
mod service_manager;
