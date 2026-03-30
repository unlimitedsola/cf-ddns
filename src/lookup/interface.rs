use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::getifaddrs::{AddressFlags, InterfaceFlags, getifaddrs};
use anyhow::{Context, Result, bail, ensure};

use crate::lookup::LookupSpec;

#[derive(Clone, Copy)]
enum IpFamily {
    V4,
    V6,
}

impl fmt::Display for IpFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            IpFamily::V4 => "IPv4",
            IpFamily::V6 => "IPv6",
        })
    }
}

pub struct InterfaceLookup {
    interface: String,
}

impl InterfaceLookup {
    pub fn new(interface: String) -> Result<Self> {
        let interface = interface.trim().to_owned();
        ensure!(
            !interface.is_empty(),
            "interface provider requires a non-empty `interface` name"
        );
        Ok(Self { interface })
    }

    fn lookup_ip<T>(
        &self,
        family: IpFamily,
        extract: impl Fn(IpAddr) -> Option<T>,
        is_public: impl Fn(T) -> bool,
    ) -> Result<T>
    where
        T: Copy + fmt::Display + Eq,
    {
        let mut found_interface = false;
        let mut candidates = Vec::new();

        for interface in getifaddrs().context("failed to enumerate network interfaces")? {
            if interface.name != self.interface {
                continue;
            }

            found_interface = true;

            if !interface.flags.contains(InterfaceFlags::UP)
                || interface.flags.contains(InterfaceFlags::LOOPBACK)
            {
                continue;
            }

            for addr_entry in &interface.addresses {
                if addr_entry.flags.contains(AddressFlags::TEMPORARY)
                    || addr_entry.flags.contains(AddressFlags::DEPRECATED)
                {
                    continue;
                }
                let Some(ip) = extract(addr_entry.address) else {
                    continue;
                };
                if !candidates.contains(&ip) {
                    candidates.push(ip);
                };
            }
        }

        ensure!(found_interface, "interface `{}` not found", self.interface);

        if let Some(addr) = candidates.iter().copied().find(|a| is_public(*a)) {
            return Ok(addr);
        }

        if candidates.is_empty() {
            bail!(
                "interface `{}` has no active {} address",
                self.interface,
                family
            );
        }

        let found = candidates
            .into_iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "interface `{}` has no public {} address (found: {found})",
            self.interface,
            family
        );
    }
}

impl LookupSpec for InterfaceLookup {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        self.lookup_ip(
            IpFamily::V4,
            |ip| match ip {
                IpAddr::V4(addr) => Some(addr),
                IpAddr::V6(_) => None,
            },
            is_public_ipv4,
        )
    }

    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        self.lookup_ip(
            IpFamily::V6,
            |ip| match ip {
                IpAddr::V4(_) => None,
                IpAddr::V6(addr) => Some(addr),
            },
            is_public_ipv6,
        )
    }
}

pub(crate) fn is_public_ipv4(addr: Ipv4Addr) -> bool {
    !matches!(
        addr.octets(),
        [0, _, _, _]
            | [10, _, _, _]
            | [100, 64..=127, _, _]
            | [127, _, _, _]
            | [169, 254, _, _]
            | [172, 16..=31, _, _]
            | [192, 0, 0, _]
            | [192, 0, 2, _]
            | [192, 168, _, _]
            | [198, 18..=19, _, _]
            | [198, 51, 100, _]
            | [203, 0, 113, _]
            | [224..=255, _, _, _]
    )
}

pub(crate) fn is_public_ipv6(addr: Ipv6Addr) -> bool {
    let [first, second, ..] = addr.segments();

    !addr.is_unspecified()
        && !addr.is_loopback()
        && !addr.is_multicast()
        && (first & 0xfe00) != 0xfc00
        && (first & 0xffc0) != 0xfe80
        && (first & 0xffc0) != 0xfec0
        && !(first == 0x2001 && second == 0x0db8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_interface_name_errors() {
        assert!(InterfaceLookup::new("   ".to_owned()).is_err());
    }

    // --- is_public_ipv4 ---

    #[test]
    fn public_ipv4_accepted() {
        assert!(is_public_ipv4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(is_public_ipv4(Ipv4Addr::new(1, 1, 1, 1)));
        assert!(is_public_ipv4(Ipv4Addr::new(93, 184, 216, 34)));
    }

    #[test]
    fn private_ipv4_rejected() {
        assert!(!is_public_ipv4(Ipv4Addr::new(0, 0, 0, 1))); // 0.0.0.0/8
        assert!(!is_public_ipv4(Ipv4Addr::new(10, 0, 0, 1))); // 10/8
        assert!(!is_public_ipv4(Ipv4Addr::new(100, 64, 0, 1))); // CGNAT 100.64/10
        assert!(!is_public_ipv4(Ipv4Addr::new(100, 127, 255, 1))); // CGNAT upper bound
        assert!(!is_public_ipv4(Ipv4Addr::new(127, 0, 0, 1))); // loopback
        assert!(!is_public_ipv4(Ipv4Addr::new(169, 254, 1, 1))); // link-local
        assert!(!is_public_ipv4(Ipv4Addr::new(172, 16, 0, 1))); // 172.16/12 lower
        assert!(!is_public_ipv4(Ipv4Addr::new(172, 31, 0, 1))); // 172.16/12 upper
        assert!(!is_public_ipv4(Ipv4Addr::new(192, 0, 0, 1))); // IETF protocol
        assert!(!is_public_ipv4(Ipv4Addr::new(192, 0, 2, 1))); // TEST-NET-1
        assert!(!is_public_ipv4(Ipv4Addr::new(192, 168, 1, 1))); // 192.168/16
        assert!(!is_public_ipv4(Ipv4Addr::new(198, 18, 0, 1))); // benchmarking
        assert!(!is_public_ipv4(Ipv4Addr::new(198, 51, 100, 1))); // TEST-NET-2
        assert!(!is_public_ipv4(Ipv4Addr::new(203, 0, 113, 1))); // TEST-NET-3
        assert!(!is_public_ipv4(Ipv4Addr::new(224, 0, 0, 1))); // multicast
        assert!(!is_public_ipv4(Ipv4Addr::new(255, 255, 255, 255))); // broadcast
    }

    // --- is_public_ipv6 ---

    #[test]
    fn public_ipv6_accepted() {
        assert!(is_public_ipv6("2606:4700:4700::1111".parse().unwrap()));
        assert!(is_public_ipv6("2001:4860:4860::8888".parse().unwrap()));
        assert!(is_public_ipv6("2a00:1450:4001::1".parse().unwrap()));
    }

    #[test]
    fn private_ipv6_rejected() {
        assert!(!is_public_ipv6(Ipv6Addr::UNSPECIFIED)); // ::
        assert!(!is_public_ipv6(Ipv6Addr::LOCALHOST)); // ::1
        assert!(!is_public_ipv6("ff02::1".parse().unwrap())); // multicast
        assert!(!is_public_ipv6("fe80::1".parse().unwrap())); // link-local
        assert!(!is_public_ipv6("fec0::1".parse().unwrap())); // site-local
        assert!(!is_public_ipv6("fc00::1".parse().unwrap())); // ULA fc::/7 lower
        assert!(!is_public_ipv6("fd00::1".parse().unwrap())); // ULA fd::/7 upper
        assert!(!is_public_ipv6("2001:db8::1".parse().unwrap())); // documentation
    }
}
