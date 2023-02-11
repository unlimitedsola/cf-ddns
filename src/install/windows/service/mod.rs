use anyhow::Result;
use windows::Win32::System::Services;

pub use sc_handle::ScHandle;
pub use service_config::ServiceConfig;
pub use service_manager::ServiceManager;

mod sc_handle;
mod service_config;
mod service_manager;

/// A created or queried Service
pub struct Service {
    handle: ScHandle,
}

impl Service {
    pub fn new(handle: ScHandle) -> Self {
        Service { handle }
    }

    pub fn delete(self) -> Result<()> {
        Ok(unsafe { Services::DeleteService(self.handle.raw_handle()) }.ok()?)
    }
}
