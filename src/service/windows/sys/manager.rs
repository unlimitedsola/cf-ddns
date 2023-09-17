use anyhow::Result;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Services;
use windows::Win32::System::Services::{
    SC_MANAGER_ALL_ACCESS, SERVICES_ACTIVE_DATABASEW, SERVICE_ALL_ACCESS, SERVICE_ERROR_IGNORE,
    SERVICE_START_TYPE, SERVICE_WIN32_OWN_PROCESS,
};

use crate::service::windows::sys::{ScHandle, Service};

/// Service Control Manager for registering and querying services
pub struct ServiceManager {
    handle: ScHandle,
}

impl ServiceManager {
    /// Open the local computer's service control manager
    ///
    /// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-openscmanagerw
    pub fn local_computer() -> Result<Self> {
        let handle = unsafe {
            let handle = Services::OpenSCManagerW(
                // null or empty string means local computer
                PCWSTR::null(),
                SERVICES_ACTIVE_DATABASEW,
                // golang's implementation hardcoded SC_MANAGER_ALL_ACCESS for simplicity
                SC_MANAGER_ALL_ACCESS,
            )?;
            ScHandle::new(handle)
        };
        Ok(ServiceManager { handle })
    }

    pub fn open_service(&self, name: &str) -> Result<Service> {
        let handle = unsafe {
            let handle = Services::OpenServiceW(
                self.handle.raw_handle(),
                &HSTRING::from(name),
                // golang's implementation hardcoded SERVICE_ALL_ACCESS for simplicity
                SERVICE_ALL_ACCESS,
            )?;
            ScHandle::new(handle)
        };
        Ok(Service::new(handle))
    }
}

/// Config holder for creating a new service
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-createservicew
pub struct ServiceCreateConfig<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub description: &'a str,
    pub start_type: SERVICE_START_TYPE,
    /// Can also have arguments
    pub command: &'a str,
}

impl ServiceManager {
    /// Create a new service
    ///
    /// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-createservicew
    pub fn create_service(&self, config: ServiceCreateConfig) -> Result<Service> {
        let handle = unsafe {
            let handle = Services::CreateServiceW(
                self.handle.raw_handle(),
                &HSTRING::from(config.name),
                &HSTRING::from(config.display_name),
                // golang's implementation hardcoded SERVICE_ALL_ACCESS for simplicity
                SERVICE_ALL_ACCESS,
                // currently we only support SERVICE_WIN32_OWN_PROCESS
                // and it is enough for our use case
                SERVICE_WIN32_OWN_PROCESS,
                config.start_type,
                // failure to start the service should not prevent the system from booting
                SERVICE_ERROR_IGNORE,
                &HSTRING::from(config.command),
                // nonsense
                PCWSTR::null(),
                None,
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
            )?;
            ScHandle::new(handle)
        };
        let service = Service::new(handle);
        service.update_description(config.description)?;
        Ok(service)
    }
}
