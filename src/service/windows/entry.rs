use anyhow::Result;
use windows::core::PWSTR;

use crate::service::windows::sys::service_dispatcher;

pub fn service_entry() -> Result<()> {
    let mut empty_str: [u16; 1] = [0; 1];
    service_dispatcher::start(
        PWSTR::from_raw(&mut empty_str as *mut _),
        Some(ffi_service_entry),
    )
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
