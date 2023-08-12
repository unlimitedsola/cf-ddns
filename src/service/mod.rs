#[cfg(windows)]
pub use self::windows::install;
#[cfg(windows)]
pub use self::windows::is_in_windows_service;
#[cfg(windows)]
pub use self::windows::run_as_service;
#[cfg(windows)]
pub use self::windows::uninstall;

#[cfg(windows)]
mod windows;

const SERVICE_NAME: &str = "cf-ddns";
const SERVICE_DISPLAY_NAME: &str = "Cloudflare DDNS";

const SERVICE_DESCRIPTION: &str =
    "Updates Cloudflare DNS records with the current public IP address.";
