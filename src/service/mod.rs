#[cfg(windows)]
pub use self::windows::is_in_windows_service;
#[cfg(windows)]
pub use self::windows::service_entry;

#[cfg(windows)]
mod windows;
