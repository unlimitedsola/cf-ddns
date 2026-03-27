use std::net::IpAddr;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(all(unix, not(target_os = "linux")))]
mod unix;
#[cfg(windows)]
mod windows;

/// Flags representing the link-level state of a network interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterfaceFlags(u32);

impl InterfaceFlags {
    pub const UP: Self = Self(0x1);
    pub const LOOPBACK: Self = Self(0x8);

    fn empty() -> Self {
        Self(0)
    }

    fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Flags describing properties of a single IP address assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddressFlags(u32);

impl AddressFlags {
    /// RFC 4941 temporary (privacy-extension) address; not suitable for inbound connections.
    pub const TEMPORARY: Self = Self(0x1);
    /// Address whose preferred lifetime has expired; the OS will not use it for new connections.
    pub const DEPRECATED: Self = Self(0x2);

    fn empty() -> Self {
        Self(0)
    }

    fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

pub struct InterfaceAddress {
    pub address: IpAddr,
    pub flags: AddressFlags,
}

pub struct Interface {
    pub name: String,
    /// Link-level flags (UP, LOOPBACK).
    pub flags: InterfaceFlags,
    /// All IP addresses assigned to this interface.
    pub addresses: Vec<InterfaceAddress>,
}

/// Returns all network interfaces that have at least one IP address assigned.
pub fn getifaddrs() -> std::io::Result<Vec<Interface>> {
    #[cfg(target_os = "linux")]
    {
        linux::get_interfaces()
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        unix::get_interfaces()
    }
    #[cfg(windows)]
    {
        windows::get_interfaces()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn loopback_interface_exists() {
        let interfaces = getifaddrs().unwrap();
        let loopback = interfaces.iter().find(|i| {
            i.flags.contains(InterfaceFlags::LOOPBACK)
                && i.addresses
                    .iter()
                    .any(|a| a.address == IpAddr::V4(Ipv4Addr::LOCALHOST))
        });
        assert!(loopback.is_some(), "No loopback IPv4 interface found");
    }

    #[test]
    fn non_loopback_interface_exists() {
        let interfaces = getifaddrs().unwrap();
        let non_loopback = interfaces
            .iter()
            .find(|i| !i.flags.contains(InterfaceFlags::LOOPBACK));
        assert!(non_loopback.is_some(), "No non-loopback interface found");
    }
}
