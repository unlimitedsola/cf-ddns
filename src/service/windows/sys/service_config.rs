use windows::Win32::System::Services::{ENUM_SERVICE_TYPE, SERVICE_ERROR, SERVICE_START_TYPE};

/// Service config holder
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-createservicew
pub struct ServiceConfig<'a> {
    pub name: &'a str,
    pub display_name: &'a str,
    pub service_type: ENUM_SERVICE_TYPE,
    pub start_type: SERVICE_START_TYPE,
    pub error_control: SERVICE_ERROR,
    /// Can also have arguments
    pub command: &'a str,
}
