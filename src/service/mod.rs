use cfg_if::cfg_if;

const SERVICE_NAME: &str = "cf-ddns";
const SERVICE_DISPLAY_NAME: &str = "Cloudflare DDNS";

const SERVICE_DESCRIPTION: &str =
    "Updates Cloudflare DNS records with the current public IP address.";

cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use self::windows::is_in_windows_service;
        pub use self::windows::run_as_service;
    } else {
        mod default;
    }
}
