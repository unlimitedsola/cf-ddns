//! A crude wrapper to make service entry point more Rustic.

use std::sync::{Mutex, MutexGuard};

use anyhow::{Context, Result, bail};
use futures::channel::oneshot;
use futures::channel::oneshot::{Receiver, Sender};
use tracing::{error, info};
use windows::Win32::System::Services;
use windows::Win32::System::Services::{
    SERVICE_ACCEPT_STOP, SERVICE_RUNNING, SERVICE_STATUS, SERVICE_WIN32_OWN_PROCESS,
};
use windows::core::{HSTRING, PWSTR};

use crate::service::windows::sys::control::{register, start};
use crate::service::windows::sys::helper::parse_service_entry_arguments;

fn running_service() -> MutexGuard<'static, Option<RunningService>> {
    static SERVICE: Mutex<Option<RunningService>> = Mutex::new(None);
    SERVICE.lock().unwrap()
}

struct RunningService {
    name: String,
    entry: ServiceMain,
    cancel: Option<Sender<()>>,
}

type ServiceMain = fn(args: Vec<String>, cancel: Receiver<()>) -> Result<()>;

pub fn run(name: &str, entry: ServiceMain) -> Result<()> {
    {
        let mut svc = running_service();
        if let Some(ref svc) = *svc {
            bail!("Service '{}' is already running.", svc.name);
        }

        *svc = Some(RunningService {
            name: name.to_owned(),
            entry,
            cancel: None,
        });
    }
    start(name, ffi_service_entry)
}

pub extern "system" fn ffi_service_entry(argc: u32, argv: *mut PWSTR) {
    let args = unsafe { parse_service_entry_arguments(argc, argv) };
    run_service(args).unwrap()
}

fn run_service(args: Vec<String>) -> Result<()> {
    let (name, entry, cancel) = {
        let mut svc = running_service();
        match *svc {
            Some(ref mut svc) => {
                let (tx, rx) = oneshot::channel::<()>();
                svc.cancel = Some(tx);
                (svc.name.clone(), svc.entry, rx)
            }
            None => bail!("Service is not running."),
        }
    };
    let w_name = HSTRING::from(name);
    let handle = register(&w_name, ffi_control_handler).context("registering service handle")?;
    handle
        .set_status(SERVICE_STATUS {
            dwServiceType: SERVICE_WIN32_OWN_PROCESS,
            dwCurrentState: SERVICE_RUNNING,
            dwControlsAccepted: SERVICE_ACCEPT_STOP,
            ..SERVICE_STATUS::default()
        })
        .context("updating service status to RUNNING")?;
    let result = entry(args, cancel);
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
        let cancel = running_service().as_mut().and_then(|svc| svc.cancel.take());
        if let Some(cancel) = cancel
            && let Err(e) = cancel.send(())
        {
            error!("Failed to send stop signal to service worker: {e:?}");
        }
    }
}
