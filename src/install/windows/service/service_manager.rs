use anyhow::Result;
use windows::core::PCWSTR;
use windows::Win32::System::Services;
use windows::Win32::System::Services::SERVICES_ACTIVE_DATABASEW;

use crate::install::windows::service::ScHandle;
use crate::install::windows::service::Service;
use crate::install::windows::service::ServiceConfig;

/// Service Control Manager for registering and querying services
pub struct ServiceManager {
    handle: ScHandle,
}

impl ServiceManager {
    /// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-openscmanagerw
    pub fn local_computer(access_flag: u32) -> Result<Self> {
        let handle = unsafe {
            let handle = Services::OpenSCManagerW(
                // current machine
                PCWSTR::null(),
                SERVICES_ACTIVE_DATABASEW,
                access_flag,
            )?;
            ScHandle::new(handle)
        };
        Ok(ServiceManager { handle })
    }

    /// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-createservicew
    pub fn create_service(&self, config: ServiceConfig, access_flag: u32) -> Result<Service> {
        let handle = unsafe {
            let handle = Services::CreateServiceW(
                self.handle.raw_handle(),
                config.name,
                config.display_name,
                access_flag,
                config.service_type,
                config.start_type,
                config.error_control,
                config.binary_path_name,
                PCWSTR::null(),
                None,
                config.dependencies,
                config.service_start_name,
                config.password,
            )?;
            ScHandle::new(handle)
        };
        Ok(Service::new(handle))
    }

    pub fn open_service(&self, name: PCWSTR, access_flag: u32) -> Result<Service> {
        let handle = unsafe {
            let handle = Services::OpenServiceW(self.handle.raw_handle(), name, access_flag)?;
            ScHandle::new(handle)
        };
        Ok(Service::new(handle))
    }
}
