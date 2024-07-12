use windows::Win32::System::Services;
use windows::Win32::System::Services::SC_HANDLE;

/// A handle wrapper for holding underlying [SC_HANDLE].
///
/// [ScHandle] implements [Drop] for automatically closing unused handles.
/// However, incorrect implementation may cause the underlying handle to get
/// invalidated before being dropped, or still holding the underlying handle
/// after this wrapper gets dropped.
pub struct ScHandle(SC_HANDLE);

impl ScHandle {
    /// Creates a wrapper for underlying [SC_HANDLE].
    /// Make sure to check [SC_HANDLE::is_invalid] before calling.
    pub unsafe fn new(handle: SC_HANDLE) -> Self {
        ScHandle(handle)
    }

    /// Returns the underlying [SC_HANDLE].
    /// For safety reasons, the returned handle SHOULD NOT be kept alive.
    pub fn raw_handle(&self) -> SC_HANDLE {
        self.0
    }
}

impl Drop for ScHandle {
    fn drop(&mut self) {
        unsafe { Services::CloseServiceHandle(self.0).ok().unwrap() };
    }
}
