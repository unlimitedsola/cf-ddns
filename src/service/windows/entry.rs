use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use windows::core::PWSTR;

use crate::cli::Cli;
use crate::service::windows::sys::service_control;
use crate::service::windows::sys::service_control::register;
use crate::AppContext;

pub fn service_entry() -> Result<()> {
    service_control::start(ffi_service_entry)
}

pub extern "system" fn ffi_service_entry(argc: u32, argv: *mut PWSTR) {
    let args = unsafe { parse_service_entry_arguments(argc, argv) };
    service_main(args)
}

unsafe fn parse_service_entry_arguments(argc: u32, argv: *mut PWSTR) -> Vec<String> {
    (0..argc)
        .map(|i| (*argv.offset(i as isize)).to_string().unwrap())
        .collect()
}

fn service_main(args: Vec<String>) {}

async fn service_main_async(args: Vec<String>) -> Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let app = Arc::new(AppContext::new(cli)?);
    Ok(())
}
