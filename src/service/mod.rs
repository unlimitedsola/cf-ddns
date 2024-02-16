#![cfg(feature = "service")]

#[cfg(target_os = "macos")]
pub use self::macos::*;
#[cfg(windows)]
pub use self::windows::*;

mod daemon;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
