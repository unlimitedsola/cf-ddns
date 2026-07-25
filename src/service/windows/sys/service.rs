use anyhow::{Context, Result};
use windows::Win32::System::Services;
use windows::Win32::System::Services::{SERVICE_CONFIG_DESCRIPTION, SERVICE_DESCRIPTIONW};
use windows::core::{HSTRING, PWSTR};

use crate::service::windows::sys::ScHandle;

/// A created or queried Service
pub struct Service {
    handle: ScHandle,
}

impl Service {
    pub const fn new(handle: ScHandle) -> Self {
        Service { handle }
    }

    /// Starts the service.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-startservicew>
    pub fn start(self) -> Result<()> {
        unsafe { Services::StartServiceW(self.handle.raw_handle(), None) }
            .context("Failed to start service")
    }

    /// Stops the service.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-controlservice>
    pub fn stop(&self) -> Result<()> {
        let mut status = Services::SERVICE_STATUS::default();
        unsafe {
            Services::ControlService(
                self.handle.raw_handle(),
                Services::SERVICE_CONTROL_STOP,
                &raw mut status,
            )
        }
        .context("Failed to stop service")?;
        Ok(())
    }

    /// Deletes the service from the service control manager.
    /// This should also stop the service if it is running.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-deleteservice>
    pub fn delete(self) -> Result<()> {
        unsafe { Services::DeleteService(self.handle.raw_handle()) }
            .context("Failed to delete service")
    }

    /// Updates the description of the service.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w>
    pub fn update_description(&self, desc: &str) -> Result<()> {
        let w_desc = HSTRING::from(desc);
        let desc_struct = SERVICE_DESCRIPTIONW {
            // SAFETY: we rely on that `w_str` will not be dropped before the call.
            // The following article also demonstrates this call won't take the
            // ownership of `w_str`:
            // https://learn.microsoft.com/en-us/windows/win32/services/changing-a-service-configuration
            lpDescription: PWSTR::from_raw(w_desc.as_ptr().cast_mut()),
        };
        unsafe {
            Services::ChangeServiceConfig2W(
                self.handle.raw_handle(),
                SERVICE_CONFIG_DESCRIPTION,
                Some(std::ptr::from_ref(&desc_struct).cast_mut().cast()),
            )
        }
        .context("Failed to update service description")
    }

    /// Queries the current status of the service.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-queryservicestatus>
    pub fn query_status(&self) -> Result<Services::SERVICE_STATUS> {
        let mut status = Services::SERVICE_STATUS::default();
        unsafe { Services::QueryServiceStatus(self.handle.raw_handle(), &raw mut status) }
            .context("Failed to query service status")?;
        Ok(status)
    }
}
