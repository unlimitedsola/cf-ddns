use crate::macros::cfg::{cfg_not_supported, cfg_windows};

const SERVICE_NAME: &str = "cf-ddns";
const SERVICE_DISPLAY_NAME: &str = "Cloudflare DDNS";

const SERVICE_DESCRIPTION: &str =
    "Updates Cloudflare DNS records with the current public IP address.";

cfg_windows! {
    mod windows;
    pub use self::windows::install;
    pub use self::windows::is_in_windows_service;
    pub use self::windows::run_as_service;
    pub use self::windows::uninstall;
}

cfg_not_supported! {
    use anyhow::Result;
    pub fn install() -> Result<()> { Ok(()) }
    pub fn uninstall() -> Result<()> { Ok(()) }
}
