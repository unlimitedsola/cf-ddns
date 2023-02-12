use anyhow::Result;
use windows::core::PWSTR;
use windows::Win32::System::Services;
use windows::Win32::System::Services::LPSERVICE_MAIN_FUNCTIONW;

pub fn start(service_name: PWSTR, service_entry: LPSERVICE_MAIN_FUNCTIONW) -> Result<()> {
    let entry_table = &[
        Services::SERVICE_TABLE_ENTRYW {
            lpServiceName: service_name,
            lpServiceProc: service_entry,
        },
        Services::SERVICE_TABLE_ENTRYW::default(),
    ];
    unsafe { Services::StartServiceCtrlDispatcherW(entry_table.as_ptr()).ok()? };
    Ok(())
}
