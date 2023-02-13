use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use anyhow::{Error, Result};
use windows::Win32::Foundation::STATUS_INFO_LENGTH_MISMATCH;
use windows::Win32::System::Threading;
use windows::Win32::System::Threading::{
    GetCurrentProcess, ProcessBasicInformation, PROCESS_BASIC_INFORMATION,
};
use windows::Win32::System::WindowsProgramming::{
    NtQuerySystemInformation, SystemProcessInformation, SYSTEM_PROCESS_INFORMATION,
};
use Threading::NtQueryInformationProcess;

/// Ref: https://cs.opensource.google/go/x/sys/+/refs/tags/v0.5.0:windows/svc/security.go;l=69
/// Ref: https://github.com/dotnet/runtime/blob/36bf84fc4a89209f4fdbc1fc201e81afd8be49b0/src/libraries/Microsoft.Extensions.Hosting.WindowsServices/src/WindowsServiceHelpers.cs
pub fn is_in_windows_service() -> Result<bool> {
    let is_in_service = unsafe {
        let cur_process = current_process_info()?;
        // Reserved3 is actually InheritedFromUniqueProcessId as per the MS documentation
        // https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess
        let parent_process = find_system_process(cur_process.Reserved3 as u32)?;

        parent_process.session_id == 0
            && parent_process
                .image_name
                .eq_ignore_ascii_case("services.exe")
    };
    Ok(is_in_service)
}

unsafe fn current_process_info() -> Result<PROCESS_BASIC_INFORMATION> {
    let mut res = PROCESS_BASIC_INFORMATION::default();
    NtQueryInformationProcess(
        GetCurrentProcess(),
        ProcessBasicInformation,
        &mut res as *mut _ as *mut c_void,
        size_of::<PROCESS_BASIC_INFORMATION>() as u32,
        null_mut(),
    )?;
    Ok(res)
}

struct SystemProcessInfo {
    session_id: u32,
    image_name: String,
}

unsafe fn find_system_process(pid: u32) -> Result<SystemProcessInfo> {
    // Generally, you need at least 512 KiB to fit all process info.
    let mut buf_size = 512 * 1024;
    loop {
        let layout = Layout::array::<u8>(buf_size)?;
        // SAFETY: Deallocate before returning errors
        let buf = alloc(layout);
        if buf.is_null() {
            handle_alloc_error(layout);
        }

        // If query failed with insufficient buffer size, the expected size
        // will be written into `needed`.
        let mut needed = 0;
        let res = NtQuerySystemInformation(
            SystemProcessInformation,
            buf as *mut _,
            buf_size as u32,
            &mut needed,
        );
        match res {
            Ok(_) => {
                let result = parse_and_find_system_process(pid, buf);
                dealloc(buf, layout);
                return result;
            }
            Err(e) => {
                dealloc(buf, layout);
                if e.code() == STATUS_INFO_LENGTH_MISMATCH.to_hresult() {
                    if needed != 0 {
                        // Adding more kilo bytes in case there were new processes just spawned in
                        buf_size = (needed + 1024 * 32) as usize;
                    } else {
                        // Tbh this should not happen, just double the size and try again I guess?
                        buf_size *= 2;
                    };
                } else {
                    return Err(e.into());
                }
            }
        }
    }
}

unsafe fn parse_and_find_system_process(pid: u32, buf: *mut u8) -> Result<SystemProcessInfo> {
    let mut offset = 0;
    loop {
        let info = &*(buf.offset(offset) as *const SYSTEM_PROCESS_INFORMATION);
        if info.UniqueProcessId.0 as u32 == pid {
            return Ok(SystemProcessInfo {
                session_id: info.SessionId,
                image_name: String::from_utf16(from_raw_parts(
                    info.ImageName.Buffer.as_ptr(),
                    info.ImageName.Length as usize,
                ))?,
            });
        }
        if info.NextEntryOffset == 0 {
            return Err(Error::msg(format!(
                "Cannot find the specified pid: {}",
                pid
            )));
        }
        offset += info.NextEntryOffset as isize;
    }
}

#[cfg(test)]
mod tests {
    use crate::service::windows::sys::service_helper::is_in_windows_service;

    #[test]
    fn test_should_not_in_windows_service() {
        assert!(!is_in_windows_service().unwrap())
    }
}
