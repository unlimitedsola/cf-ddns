use anyhow::Result;
use windows::core::{HSTRING, PWSTR};
use windows::Win32::System::Services;
use windows::Win32::System::Services::SERVICE_STATUS;

pub type ServiceMain = extern "system" fn(argc: u32, argv: *mut PWSTR) -> ();

/// Connects the main thread of a service process to the service control manager, which causes
/// the thread to be the service control dispatcher thread for the calling process. This call
/// returns when the service has stopped. The process should simply terminate when the call returns.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-startservicectrldispatcherw
pub fn start(name: &HSTRING, entry: ServiceMain) -> Result<()> {
    let entry_table = &[
        Services::SERVICE_TABLE_ENTRYW {
            // If the service type is SERVICE_WIN32_OWN_PROCESS, this field
            // is ignored, but cannot be NULL.
            // Not sure why Windows is asking for a mutable string here.
            // Assuming they won't actually mutate it?
            lpServiceName: PWSTR::from_raw(name.as_ptr() as *mut _),
            lpServiceProc: Some(entry),
        },
        Services::SERVICE_TABLE_ENTRYW::default(),
    ];
    unsafe { Services::StartServiceCtrlDispatcherW(entry_table.as_ptr())? };
    Ok(())
}

/// A unique token for updating the status of the corresponding service.
#[derive(Debug, Clone, Copy)]
pub struct ServiceStatusHandle(Services::SERVICE_STATUS_HANDLE);

impl ServiceStatusHandle {
    pub fn new(handle: Services::SERVICE_STATUS_HANDLE) -> Self {
        ServiceStatusHandle(handle)
    }

    pub fn set_status(&self, status: SERVICE_STATUS) -> Result<()> {
        unsafe { Services::SetServiceStatus(self.0, &status)? };
        Ok(())
    }
}

// Underlying SERVICE_STATUS_HANDLE is thread safe.
// See remarks section for more info:
// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-setservicestatus#remarks
unsafe impl Send for ServiceStatusHandle {}

unsafe impl Sync for ServiceStatusHandle {}

type ControlHandler = extern "system" fn(control: u32) -> ();

/// Registers a service status handler to receive status control signals.
///
/// This function uses the simpler variant of the `RegisterServiceCtrlHandler` functions for simplicity.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-registerservicectrlhandlerw
pub fn register(name: &HSTRING, handler: ControlHandler) -> Result<ServiceStatusHandle> {
    let status_handle = unsafe { Services::RegisterServiceCtrlHandlerW(name, Some(handler))? };

    Ok(ServiceStatusHandle::new(status_handle))
}
