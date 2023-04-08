use core::ffi::c_void;

use anyhow::{anyhow, Result};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_CALL_NOT_IMPLEMENTED, WIN32_ERROR};
use windows::Win32::System::Services;

pub type ServiceMain = extern "system" fn(argc: u32, argv: *mut PWSTR) -> ();

/// Connects the main thread of a service process to the service control manager, which causes
/// the thread to be the service control dispatcher thread for the calling process. This call
/// returns when the service has stopped. The process should simply terminate when the call returns.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-startservicectrldispatcherw
pub fn start(service_entry: ServiceMain) -> Result<()> {
    let entry_table = &[
        Services::SERVICE_TABLE_ENTRYW {
            lpServiceName: PWSTR::null(),
            lpServiceProc: Some(service_entry),
        },
        Services::SERVICE_TABLE_ENTRYW::default(),
    ];
    unsafe { Services::StartServiceCtrlDispatcherW(entry_table.as_ptr()).ok()? };
    Ok(())
}

/// A unique token for updating the status of the corresponding service.
#[derive(Debug, Clone, Copy)]
pub struct ServiceStatusHandle(Services::SERVICE_STATUS_HANDLE);

impl ServiceStatusHandle {
    pub fn new(handle: Services::SERVICE_STATUS_HANDLE) -> Self {
        ServiceStatusHandle(handle)
    }

    pub fn set_status(&self, status: Services::SERVICE_STATUS) -> Result<()> {
        unsafe { Services::SetServiceStatus(self.0, &status).ok()? };
        Ok(())
    }
}

// Underlying SERVICE_STATUS_HANDLE is thread safe.
// See remarks section for more info:
// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-setservicestatus#remarks
unsafe impl Send for ServiceStatusHandle {}

unsafe impl Sync for ServiceStatusHandle {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceControl {
    Interrogate,
    Shutdown,
    Stop,
}

impl ServiceControl {
    pub unsafe fn from_raw(control: u32, event_type: u32, event_data: *mut c_void) -> Result<Self> {
        match control {
            Services::SERVICE_CONTROL_INTERROGATE => Ok(ServiceControl::Interrogate),
            Services::SERVICE_CONTROL_SHUTDOWN => Ok(ServiceControl::Shutdown),
            Services::SERVICE_CONTROL_STOP => Ok(ServiceControl::Stop),
            _ => Err(anyhow!(
                "Unknown service control: {} {} {:?}",
                control,
                event_type,
                event_data,
            )),
        }
    }
}

/// Registers a service status handler to receive status control signals.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-registerservicectrlhandlerw
pub fn register<F>(handler: F) -> Result<ServiceStatusHandle>
where
    F: FnMut(ServiceControl) -> WIN32_ERROR + 'static + Send,
{
    // Move closure into heap
    let heap_handler: Box<F> = Box::new(handler);
    // Leak the handler function to prevent it from dropping.
    // This is required as the leaked handler is used in `internal_status_handler`
    // which will be called from Windows service dispatcher that is outside of our control.
    // SAFETY: the leaked handler will be released in `internal_status_handler` when
    // service handle is being closed.
    let context: *mut F = Box::into_raw(heap_handler);

    let status_handle = unsafe {
        Services::RegisterServiceCtrlHandlerExW(
            PCWSTR::null(),
            Some(internal_status_handler::<F>),
            // handler function pointer is passed as context
            Some(context as *mut c_void),
        )
    };

    match status_handle {
        Ok(handle) => Ok(ServiceStatusHandle::new(handle)),
        Err(e) => {
            // SAFETY: release the handler in case of an error.
            let _: Box<F> = unsafe { Box::from_raw(context) };
            Err(e.into())
        }
    }
}

extern "system" fn internal_status_handler<F>(
    control: u32,
    event_type: u32,
    event_data: *mut c_void,
    context: *mut c_void,
) -> u32
where
    F: FnMut(ServiceControl) -> WIN32_ERROR + 'static + Send,
{
    // SAFETY: cast the context back to the handler without taking ownership (so it won't get dropped).
    let handler: &mut F = unsafe { &mut *(context as *mut F) };

    match unsafe { ServiceControl::from_raw(control, event_type, event_data) } {
        Ok(control) => {
            let result = handler(control);
            if matches!(control, ServiceControl::Stop | ServiceControl::Shutdown) {
                // SAFETY: release the handler when service handle is being closed.
                let _: Box<F> = unsafe { Box::from_raw(context as *mut F) };
            }
            result
        }
        Err(_) => ERROR_CALL_NOT_IMPLEMENTED,
    }
    .0
}
