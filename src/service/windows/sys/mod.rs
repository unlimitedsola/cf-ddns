use std::ffi::c_void;
use std::mem::size_of;
use std::{env, process};

use anyhow::Result;
use windows::Win32::System::Threading;
use windows::Win32::System::Threading::{
    GetCurrentProcess, ProcessBasicInformation, PROCESS_BASIC_INFORMATION,
};

pub use sc_handle::ScHandle;
pub use service::Service;
pub use service_config::ServiceConfig;
pub use service_manager::ServiceManager;

mod sc_handle;
mod service;
mod service_config;
mod service_dispatcher;
mod service_helper;
mod service_manager;
