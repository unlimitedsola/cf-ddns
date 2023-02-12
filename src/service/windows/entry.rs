use windows::core::PWSTR;

extern "system" fn service_entry(argc: u32, argv: *mut PWSTR) {
    let args = unsafe { parse_service_entry_arguments(argc, argv) };
    service_main(args)
}

unsafe fn parse_service_entry_arguments(argc: u32, argv: *mut PWSTR) -> Vec<String> {
    (0..argc)
        .map(|i| (*argv.offset(i as isize)).to_string().unwrap())
        .collect()
}

fn service_main(args: Vec<String>) {}
