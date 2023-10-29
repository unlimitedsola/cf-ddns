use anyhow::Result;
use windows::core::{HSTRING, PWSTR};
use windows::Win32::System::Services;
use windows::Win32::System::Services::{SERVICE_CONFIG_DESCRIPTION, SERVICE_DESCRIPTIONW};

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
        unsafe { Services::DeleteService(self.handle.raw_handle())? };
        Ok(())
    }

    pub fn update_description(&self, desc: &str) -> Result<()> {
        unsafe {
            Services::ChangeServiceConfig2W(
                self.handle.raw_handle(),
                SERVICE_CONFIG_DESCRIPTION,
                Some(&SERVICE_DESCRIPTIONW {
                    lpDescription: PWSTR::from_raw(HSTRING::from(desc).as_ptr() as *mut _),
                } as *const _ as *mut _),
            )?;
        }
        Ok(())
    }
}
