//! A crude wrapper to make service entry point more Rustic.

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use futures::channel::oneshot;
use futures::channel::oneshot::{Receiver, Sender};
use tracing::{error, info};
use windows::core::{HSTRING, PWSTR};
use windows::Win32::System::Services;
use windows::Win32::System::Services::{
    SERVICE_ACCEPT_STOP, SERVICE_RUNNING, SERVICE_STATUS, SERVICE_WIN32_OWN_PROCESS,
};

use crate::service::windows::sys::control::{register, start};
use crate::service::windows::sys::helper::parse_service_entry_arguments;

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
    let r = run_service(argc, argv);
    if let Err(e) = r {
        error!("Error occurred while starting the service: {e:#?}");
    }
}

fn run_service(argc: u32, argv: *mut PWSTR) -> Result<()> {
    let mut args = unsafe { parse_service_entry_arguments(argc, argv) };
    // Remove the first argument, which is the executable name.
    args.remove(0);
    let (tx, rx) = oneshot::channel::<()>();
    let (name, entry) = {
        let mut service = SERVICE.lock().unwrap();
        service.cancel = Some(tx);
        (
            service.name.as_ref().unwrap().clone(),
            service.entry.as_ref().unwrap().clone(),
        )
    };
    let handle = register(&name, ffi_control_handler).context("registering service handle")?;
    handle
        .set_status(SERVICE_STATUS {
            dwServiceType: SERVICE_WIN32_OWN_PROCESS,
            dwCurrentState: SERVICE_RUNNING,
            dwControlsAccepted: SERVICE_ACCEPT_STOP,
            ..SERVICE_STATUS::default()
        })
        .context("updating service status to RUNNING")?;
    let result = entry(args, rx);
    handle
        .set_status(SERVICE_STATUS {
            dwServiceType: SERVICE_WIN32_OWN_PROCESS,
            dwCurrentState: Services::SERVICE_STOPPED,
            dwWin32ExitCode: result.as_ref().map_or(1, |_| 0),
            ..SERVICE_STATUS::default()
        })
        .context("updating service status to STOPPED")?;
    result
}

pub extern "system" fn ffi_control_handler(control: u32) {
    info!("Received service control signal.");
    if control == Services::SERVICE_CONTROL_STOP {
        info!("Sending stop signal to service worker...");
        let tx = SERVICE.lock().unwrap().cancel.take().unwrap();
        tx.send(()).unwrap();
    }
}
