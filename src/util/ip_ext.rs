//! Backports of unstable standard library IP address classification methods.
//!
//! The helper functions in this module are copied from the unstable standard library
//! implementation of `is_global` for IP addresses:
//! - `std::net::Ipv4Addr::is_global`: <https://doc.rust-lang.org/stable/std/net/struct.Ipv4Addr.html#method.is_global>
//! - `std::net::Ipv6Addr::is_global`: <https://doc.rust-lang.org/stable/std/net/struct.Ipv6Addr.html#method.is_global>

use std::net::{Ipv4Addr, Ipv6Addr};

/// Returns `true` if the address is a globally routable unicast address.
pub const fn is_global_ipv4(addr: Ipv4Addr) -> bool {
    !(addr.octets()[0] == 0 // "This network"
        || addr.is_private()
        || is_shared_v4(addr)
        || addr.is_loopback()
        || addr.is_link_local()
        // Addresses reserved for future protocols (`192.0.0.0/24`)
        // .9 and .10 are documented as globally reachable so they're excluded
        || (
            addr.octets()[0] == 192 && addr.octets()[1] == 0 && addr.octets()[2] == 0
            && addr.octets()[3] != 9 && addr.octets()[3] != 10
        )
        || is_documentation_v4(addr)
        || is_benchmarking_v4(addr)
        || is_reserved_v4(addr)
        || is_broadcast_v4(addr))
}

/// Returns `true` if the address is a globally routable unicast address.
pub const fn is_global_ipv6(addr: &Ipv6Addr) -> bool {
    !(addr.is_unspecified()
        || addr.is_loopback()
        // IPv4-mapped Address (`::ffff:0:0/96`)
        || matches!(addr.segments(), [0, 0, 0, 0, 0, 0xffff, _, _])
        // IPv4-IPv6 Translation (`64:ff9b:1::/48`)
        || matches!(addr.segments(), [0x64, 0xff9b, 1, _, _, _, _, _])
        // Discard-Only Address Block (`100::/64`)
        || matches!(addr.segments(), [0x100, 0, 0, 0, _, _, _, _])
        // IETF Protocol Assignments (`2001::/23`)
        || (matches!(addr.segments(), [0x2001, 0..=0x1ff, _, _, _, _, _, _])
            && !(
                // Port Control Protocol Anycast (`2001:1::1`)
                u128::from_be_bytes(addr.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0001
                // Traversal Using Relays around NAT Anycast (`2001:1::2`)
                || u128::from_be_bytes(addr.octets()) == 0x2001_0001_0000_0000_0000_0000_0000_0002
                // AMT (`2001:3::/32`)
                || matches!(addr.segments(), [0x2001, 3, _, _, _, _, _, _])
                // AS112-v6 (`2001:4:112::/48`)
                || matches!(addr.segments(), [0x2001, 4, 0x112, _, _, _, _, _])
                // ORCHIDv2 (`2001:20::/28`)
                // Drone Remote ID Protocol Entity Tags (DETs) Prefix (`2001:30::/28`)
                || matches!(addr.segments(), [0x2001, 0x20..=0x3F, _, _, _, _, _, _])
            ))
        // 6to4 (`2002::/16`) – it's not explicitly documented as globally reachable, IANA says N/A
        || matches!(addr.segments(), [0x2002, _, _, _, _, _, _, _])
        || is_documentation_v6(addr)
        // Segment Routing (SRv6) SIDs (`5f00::/16`)
        || matches!(addr.segments(), [0x5f00, ..])
        || is_unique_local_v6(addr)
        || is_unicast_link_local_v6(addr))
}

// --- IPv4 Helpers ---

const fn is_shared_v4(addr: Ipv4Addr) -> bool {
    addr.octets()[0] == 100 && (addr.octets()[1] & 0b1100_0000 == 0b0100_0000)
}

const fn is_benchmarking_v4(addr: Ipv4Addr) -> bool {
    addr.octets()[0] == 198 && (addr.octets()[1] & 0xfe) == 18
}

const fn is_documentation_v4(addr: Ipv4Addr) -> bool {
    matches!(
        addr.octets(),
        [192, 0, 2, _] | [198, 51, 100, _] | [203, 0, 113, _]
    )
}

const fn is_reserved_v4(addr: Ipv4Addr) -> bool {
    addr.octets()[0] & 0xf0 == 0xf0 && !is_broadcast_v4(addr)
}

const fn is_broadcast_v4(addr: Ipv4Addr) -> bool {
    matches!(addr.octets(), [255, 255, 255, 255])
}

// --- IPv6 Helpers ---

const fn is_documentation_v6(addr: &Ipv6Addr) -> bool {
    matches!(
        addr.segments(),
        [0x2001, 0xdb8, ..] | [0x3fff, 0..=0x0fff, ..]
    )
}

const fn is_unique_local_v6(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

const fn is_unicast_link_local_v6(addr: &Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}
