use anyhow::Result;
use windows::Win32::System::Services;

use crate::service::windows::sys::ScHandle;

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
