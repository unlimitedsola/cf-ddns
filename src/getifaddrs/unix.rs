use super::{AddressFlags, Interface, InterfaceAddress, InterfaceFlags};
use std::ffi::CStr;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn get_interfaces() -> io::Result<Vec<Interface>> {
    let iter = InterfaceIterator::new()?;
    let mut interfaces: Vec<Interface> = Vec::new();
    for entry in iter {
        if let Some(iface) = interfaces.iter_mut().find(|i| i.name == entry.name) {
            iface.addresses.push(entry.addr);
        } else {
            interfaces.push(Interface {
                name: entry.name,
                flags: entry.link_flags,
                addresses: vec![entry.addr],
            });
        }
    }
    Ok(interfaces)
}

struct RawEntry {
    name: String,
    link_flags: InterfaceFlags,
    addr: InterfaceAddress,
}

struct InterfaceIterator {
    ifaddrs: *mut libc::ifaddrs,
    current: *mut libc::ifaddrs,
    #[cfg(target_os = "macos")]
    sock6: libc::c_int,
}

impl InterfaceIterator {
    pub fn new() -> io::Result<Self> {
        let mut ifaddrs: *mut libc::ifaddrs = std::ptr::null_mut();
        if unsafe { libc::getifaddrs(&mut ifaddrs) } != 0 {
            return Err(io::Error::last_os_error());
        }

        #[cfg(target_os = "macos")]
        let sock6 = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, 0) };

        Ok(Self {
            ifaddrs,
            current: ifaddrs,
            #[cfg(target_os = "macos")]
            sock6,
        })
    }
}

impl Iterator for InterfaceIterator {
    type Item = RawEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current.is_null() {
                return None;
            }
            let ifaddr = unsafe { &*self.current };
            self.current = ifaddr.ifa_next;

            let Some(addr) = (unsafe { ifaddr.ifa_addr.as_ref() }) else {
                continue;
            };

            let address: IpAddr = match addr.sa_family as i32 {
                libc::AF_INET => {
                    let sa_in = addr as *const _ as *const libc::sockaddr_in;
                    Ipv4Addr::from(unsafe { (*sa_in).sin_addr.s_addr }.to_ne_bytes()).into()
                }
                libc::AF_INET6 => {
                    let sa_in6 = addr as *const _ as *const libc::sockaddr_in6;
                    Ipv6Addr::from(unsafe { (*sa_in6).sin6_addr.s6_addr }).into()
                }
                _ => continue,
            };

            let raw = ifaddr.ifa_flags as usize;
            let mut link_flags = InterfaceFlags::empty();
            if raw & (libc::IFF_UP as usize) != 0 && raw & (libc::IFF_RUNNING as usize) != 0 {
                link_flags = link_flags.with(InterfaceFlags::UP);
            }
            if raw & (libc::IFF_LOOPBACK as usize) != 0 {
                link_flags = link_flags.with(InterfaceFlags::LOOPBACK);
            }

            let mut addr_flags = AddressFlags::empty();
            #[cfg(target_os = "macos")]
            if let IpAddr::V6(_) = address {
                if self.sock6 >= 0 {
                    addr_flags = addr_flags.with(read_ipv6_addr_flags(
                        self.sock6,
                        ifaddr.ifa_name,
                        addr as *const _ as *const libc::sockaddr_in6,
                    ));
                }
            }

            let name = unsafe { CStr::from_ptr(ifaddr.ifa_name) }
                .to_string_lossy()
                .into_owned();

            return Some(RawEntry {
                name,
                link_flags,
                addr: InterfaceAddress {
                    address,
                    flags: addr_flags,
                },
            });
        }
    }
}

impl Drop for InterfaceIterator {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.ifaddrs) };
        #[cfg(target_os = "macos")]
        if self.sock6 >= 0 {
            unsafe { libc::close(self.sock6) };
        }
    }
}

// Constants for macOS can be found in the XNU source code, in in6_var.h:
// https://github.com/apple-oss-distributions/xnu/blob/main/bsd/netinet6/in6_var.h

// macOS: per-address IPv6 flags via ioctl(SIOCGIFAFLAG_IN6).
#[cfg(target_os = "macos")]
const SIOCGIFAFLAG_IN6: libc::c_ulong = 0xC120_6949;

#[cfg(target_os = "macos")]
const IN6_IFF_DEPRECATED: libc::c_int = 0x0010;

#[cfg(target_os = "macos")]
const IN6_IFF_TEMPORARY: libc::c_int = 0x0080;

#[cfg(target_os = "macos")]
fn read_ipv6_addr_flags(
    sock: libc::c_int,
    name: *const libc::c_char,
    sa_in6: *const libc::sockaddr_in6,
) -> AddressFlags {
    let mut req: libc::in6_ifreq = unsafe { std::mem::zeroed() };
    let raw = unsafe {
        // Copy interface name (null-terminated, leave final byte as zero).
        let name_bytes = CStr::from_ptr(name).to_bytes();
        let copy_len = name_bytes.len().min(req.ifr_name.len() - 1);
        for (dst, &src) in req.ifr_name.iter_mut().zip(&name_bytes[..copy_len]) {
            *dst = src as libc::c_char;
        }
        // Set the address in the union so the kernel knows which address to query.
        req.ifr_ifru.ifru_addr = *sa_in6;
        if libc::ioctl(sock, SIOCGIFAFLAG_IN6, &mut req as *mut _) < 0 {
            return AddressFlags::empty();
        }
        // After the ioctl the kernel has written the per-address flags into ifru_flags6.
        req.ifr_ifru.ifru_flags6
    };
    let mut flags = AddressFlags::empty();
    if raw & IN6_IFF_TEMPORARY != 0 {
        flags = flags.with(AddressFlags::TEMPORARY);
    }
    if raw & IN6_IFF_DEPRECATED != 0 {
        flags = flags.with(AddressFlags::DEPRECATED);
    }
    flags
}
