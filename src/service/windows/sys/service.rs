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
        let w_desc = HSTRING::from(desc);
        unsafe {
            Services::ChangeServiceConfig2W(
                self.handle.raw_handle(),
                SERVICE_CONFIG_DESCRIPTION,
                Some(&SERVICE_DESCRIPTIONW {
                    // SAFETY: we rely on that `w_str` will not be dropped before the call.
                    // The following article also demonstrates this call won't take the
                    // ownership of `w_str`:
                    // https://learn.microsoft.com/en-us/windows/win32/services/changing-a-service-configuration
                    lpDescription: PWSTR::from_raw(w_desc.as_ptr() as *mut _),
                } as *const _ as *mut _),
            )?;
        }
        Ok(())
    }
}
