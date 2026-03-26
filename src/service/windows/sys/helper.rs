use std::ffi::c_void;
use std::mem::{align_of, size_of};
use std::ptr::null_mut;

use anyhow::{Result, bail};
use windows::Wdk::System::SystemInformation::{NtQuerySystemInformation, SystemProcessInformation};
use windows::Wdk::System::Threading::{NtQueryInformationProcess, ProcessBasicInformation};
use windows::Win32::Foundation::STATUS_INFO_LENGTH_MISMATCH;
use windows::Win32::System::Threading::{GetCurrentProcess, PROCESS_BASIC_INFORMATION};
use windows::Win32::System::WindowsProgramming::SYSTEM_PROCESS_INFORMATION;
use windows::core::PWSTR;

/// Convert Windows service entry arguments to a Rust `Vec<String>`.
///
/// # Safety
///
/// The `argv` pointer must be a valid pointer to a size `argc` array of pointers to
/// null-terminated wide strings.
/// The pointers in the array must hold the safety guarantees of [`PWSTR::to_string`].
pub unsafe fn parse_service_entry_arguments(argc: u32, argv: *mut PWSTR) -> Vec<String> {
    (0..argc)
        .map(|i| unsafe {
            (*argv.offset(i as isize))
                .to_string()
                .expect("string should be valid")
        })
        .collect()
}

/// Determines if the current process is running as a Windows service.
///
/// The implementation is borrowed from golang's `x/sys/windows/svc/security.go`:
/// <https://cs.opensource.google/go/x/sys/+/refs/tags/v0.5.0:windows/svc/security.go;l=69>
/// which they also borrowed from the following .NET implementation:
/// <https://github.com/dotnet/runtime/blob/36bf84fc4a89209f4fdbc1fc201e81afd8be49b0/src/libraries/Microsoft.Extensions.Hosting.WindowsServices/src/WindowsServiceHelpers.cs>
pub fn is_in_windows_service() -> Result<bool> {
    let cur_process = current_process_info()?;
    let parent_process = find_system_process(cur_process.InheritedFromUniqueProcessId)?;
    let is_in_service = parent_process.session_id == 0
        && parent_process
            .image_name
            .eq_ignore_ascii_case("services.exe");
    Ok(is_in_service)
}

fn current_process_info() -> Result<PROCESS_BASIC_INFORMATION> {
    let mut res = PROCESS_BASIC_INFORMATION::default();
    unsafe {
        NtQueryInformationProcess(
            GetCurrentProcess(),
            ProcessBasicInformation,
            &mut res as *mut _ as *mut c_void,
            size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            null_mut(),
        )
    }
    .ok()?;
    Ok(res)
}

#[derive(Debug)]
struct SystemProcessInfo {
    session_id: u32,
    image_name: String,
}

fn find_system_process(pid: usize) -> Result<SystemProcessInfo> {
    // Generally, you need at least 512 KiB to fit all process info.
    let mut buf_size: usize = 512 * 1024;
    loop {
        let mut buf = AlignedBuf::new::<SYSTEM_PROCESS_INFORMATION>(buf_size)?;

        // If query failed with insufficient buffer size, the expected size
        // will be written into `needed`.
        let mut needed = 0;
        let res = unsafe {
            NtQuerySystemInformation(
                SystemProcessInformation,
                buf.as_mut_ptr() as *mut c_void,
                buf_size as u32,
                &mut needed,
            )
        };
        match res.ok() {
            Ok(_) => {
                return unsafe { parse_and_find_system_process(pid, buf.as_mut_ptr()) };
            }
            Err(e) => {
                if e.code() == STATUS_INFO_LENGTH_MISMATCH.to_hresult() {
                    if needed != 0 {
                        // Adding more kilo bytes in case there were new processes just spawned in
                        buf_size = (needed + 1024 * 32) as usize;
                    } else {
                        // Tbh this should not happen, just double the size and try again I guess?
                        buf_size *= 2;
                    };
                } else {
                    bail!(e);
                }
            }
        }
    }
}

unsafe fn parse_and_find_system_process(pid: usize, buf: *const u8) -> Result<SystemProcessInfo> {
    let mut offset = 0usize;
    loop {
        // SAFETY: `buf + offset` points within the buffer written by NtQuerySystemInformation.
        // We use read_unaligned because MSDN does not document that NextEntryOffset is guaranteed
        // to produce a pointer aligned for SYSTEM_PROCESS_INFORMATION.
        let info =
            unsafe { (buf.add(offset) as *const SYSTEM_PROCESS_INFORMATION).read_unaligned() };
        if info.UniqueProcessId.0 as usize == pid {
            return Ok(SystemProcessInfo {
                session_id: info.SessionId,
                image_name: unsafe { info.ImageName.Buffer.to_string()? },
            });
        }
        if info.NextEntryOffset == 0 {
            // Reached the end of the list
            bail!("Could not find process with pid {}", pid);
        }
        offset += info.NextEntryOffset as usize;
    }
}

/// A raw byte buffer with an explicit alignment, used as an FFI output buffer.
///
/// The buffer is intentionally untyped: `NtQuerySystemInformation` writes a mix of
/// `SYSTEM_PROCESS_INFORMATION` and `SYSTEM_THREAD_INFORMATION` records into it. A typed
/// `Vec<SYSTEM_PROCESS_INFORMATION>` would misrepresent the contents.
struct AlignedBuf {
    ptr: *mut u8,
    layout: std::alloc::Layout,
}

impl AlignedBuf {
    fn new<T>(bytes: usize) -> Result<Self> {
        let layout = std::alloc::Layout::from_size_align(bytes, align_of::<T>())?;
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            bail!("Failed to allocate {} bytes for process info buffer", bytes);
        }
        Ok(Self { ptr, layout })
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr, self.layout) };
    }
}

#[cfg(test)]
mod tests {
    use crate::service::windows::sys::helper::is_in_windows_service;

    #[test]
    fn test_should_not_in_windows_service() {
        assert!(!is_in_windows_service().unwrap())
    }

    #[test]
    #[ignore]
    fn mem_leak_test() {
        // maybe? dunno how to test memory leaks :P
        for _ in 0..1_000_000 {
            assert!(!is_in_windows_service().unwrap())
        }
    }
}
