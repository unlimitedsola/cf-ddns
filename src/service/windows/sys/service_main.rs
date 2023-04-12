use std::sync::{Arc, Mutex};

use anyhow::Result;
use futures::channel::oneshot;
use futures::channel::oneshot::{Receiver, Sender};
use windows::core::{HSTRING, PWSTR};
use windows::Win32::System::Services;
use windows::Win32::System::Services::{SERVICE_ACCEPT_STOP, SERVICE_RUNNING, SERVICE_STATUS};

use crate::service::windows::sys::parse_service_entry_arguments;
use crate::service::windows::sys::service_control::{register, start, ServiceStatusHandle};

static SERVICE: Mutex<RunningService> = Mutex::new(RunningService {
    name: None,
    entry: None,
    cancel: None,
});

#[derive(Default)]
struct RunningService {
    name: Option<Arc<HSTRING>>,
    entry: Option<Arc<ServiceMain>>,
    cancel: Option<Sender<()>>,
}

type ServiceMain = fn(args: Vec<String>, cancel: Receiver<()>) -> Result<()>;

pub fn run(name: &str, entry: ServiceMain) -> Result<()> {
    let name = {
        let name = Arc::new(name.into());
        let mut service = SERVICE.lock().unwrap();
        service.name = Some(Arc::clone(&name));
        service.entry = Some(Arc::new(entry));
        name
    };
    start(&name, ffi_service_entry)
}

pub extern "system" fn ffi_service_entry(argc: u32, argv: *mut PWSTR) {
    let args = unsafe { parse_service_entry_arguments(argc, argv) };
    let (tx, rx) = oneshot::channel::<()>();
    let (name, entry) = {
        let mut service = SERVICE.lock().unwrap();
        service.cancel = Some(tx);
        (
            service.name.as_ref().unwrap().clone(),
            service.entry.as_ref().unwrap().clone(),
        )
    };
    let handle = register(&name, ffi_control_handler).unwrap();
    handle
        .set_status(SERVICE_STATUS {
            dwCurrentState: SERVICE_RUNNING,
            dwControlsAccepted: SERVICE_ACCEPT_STOP,
            ..SERVICE_STATUS::default()
        })
        .unwrap();
    let result = entry(args, rx);
    handle
        .set_status(SERVICE_STATUS {
            dwCurrentState: Services::SERVICE_STOPPED,
            dwWin32ExitCode: result.map_or(1, |_| 0),
            ..SERVICE_STATUS::default()
        })
        .unwrap();
}

pub extern "system" fn ffi_control_handler(control: u32) {
    if control == Services::SERVICE_CONTROL_STOP {
        let tx = SERVICE.lock().unwrap().cancel.take().unwrap();
        tx.send(()).unwrap();
    }
}
