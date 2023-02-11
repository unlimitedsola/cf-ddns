use windows::core::PCWSTR;
use windows::Win32::System::Services::{ENUM_SERVICE_TYPE, SERVICE_ERROR, SERVICE_START_TYPE};

/// Service config holder
/// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-createservicew
pub struct ServiceConfig {
    pub name: PCWSTR,
    pub display_name: Option<PCWSTR>,
    pub service_type: ENUM_SERVICE_TYPE,
    pub start_type: SERVICE_START_TYPE,
    pub error_control: SERVICE_ERROR,
    pub binary_path_name: Option<PCWSTR>,
    pub dependencies: Option<PCWSTR>,
    pub service_start_name: Option<PCWSTR>,
    pub password: Option<PCWSTR>,
}
