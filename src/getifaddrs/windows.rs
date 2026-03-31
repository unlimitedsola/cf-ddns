use super::{AddressFlags, Interface, InterfaceAddress, InterfaceFlags};
use std::{io, net::IpAddr};
use windows::Win32::Foundation::{
    ERROR_BUFFER_OVERFLOW, ERROR_NO_DATA, ERROR_NOT_ENOUGH_MEMORY, NO_ERROR, WIN32_ERROR,
};
use windows::Win32::NetworkManagement::IpHelper::{
    ConvertInterfaceLuidToIndex, GET_ADAPTERS_ADDRESSES_FLAGS, GetAdaptersAddresses,
    GetNumberOfInterfaces, IP_ADAPTER_ADDRESSES_LH, MIB_IF_TYPE_LOOPBACK, if_indextoname,
};
use windows::Win32::NetworkManagement::Ndis::{IfOperStatusUp, NET_LUID_LH};
use windows::Win32::Networking::WinSock::{
    AF_INET, AF_INET6, IpSuffixOriginRandom, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6,
};

const IF_NAMESIZE: usize = 256;

pub fn get_interfaces() -> io::Result<Vec<Interface>> {
    let adapters = AdaptersAddresses::try_new()?;
    let mut interfaces = Vec::new();
    let mut adapter_ptr = adapters.buf.ptr;
    while !adapter_ptr.is_null() {
        let adapter = unsafe { &*adapter_ptr };

        let mut iface_flags = InterfaceFlags::empty();
        if adapter.OperStatus == IfOperStatusUp {
            iface_flags = iface_flags.with(InterfaceFlags::UP);
        }
        if adapter.IfType == MIB_IF_TYPE_LOOPBACK {
            iface_flags = iface_flags.with(InterfaceFlags::LOOPBACK);
        }

        let mut addresses = Vec::new();
        let mut unicast_ptr = adapter.FirstUnicastAddress;
        while !unicast_ptr.is_null() {
            let unicast = unsafe { &*unicast_ptr };
            if let Ok(address) = sockaddr_to_ipaddr(unicast.Address.lpSockaddr) {
                let mut addr_flags = AddressFlags::empty();
                if unicast.SuffixOrigin == IpSuffixOriginRandom {
                    addr_flags = addr_flags.with(AddressFlags::TEMPORARY);
                }
                if unicast.PreferredLifetime == 0 && matches!(address, IpAddr::V6(_)) {
                    addr_flags = addr_flags.with(AddressFlags::DEPRECATED);
                }
                addresses.push(InterfaceAddress {
                    address,
                    flags: addr_flags,
                });
            }
            unicast_ptr = unicast.Next;
        }

        if !addresses.is_empty() {
            interfaces.push(Interface {
                name: luid_to_name(adapter.Luid),
                flags: iface_flags,
                addresses,
            });
        }
        adapter_ptr = adapter.Next;
    }
    Ok(interfaces)
}

struct AdaptersAddresses {
    buf: AdapterAddressBuf,
}

struct AdapterAddressBuf {
    ptr: *mut IP_ADAPTER_ADDRESSES_LH,
    size: usize,
}

impl AdapterAddressBuf {
    fn new(bytes: usize) -> io::Result<Self> {
        let layout = std::alloc::Layout::from_size_align(
            bytes,
            std::mem::align_of::<IP_ADAPTER_ADDRESSES_LH>(),
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            Err(io::Error::new(
                io::ErrorKind::OutOfMemory,
                "Failed to allocate memory",
            ))
        } else {
            Ok(Self {
                ptr: ptr as *mut IP_ADAPTER_ADDRESSES_LH,
                size: bytes,
            })
        }
    }
}

impl Drop for AdapterAddressBuf {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::from_size_align(
            self.size,
            std::mem::align_of::<IP_ADAPTER_ADDRESSES_LH>(),
        )
        .unwrap();
        unsafe { std::alloc::dealloc(self.ptr as *mut u8, layout) };
    }
}

impl AdaptersAddresses {
    fn try_new() -> io::Result<Self> {
        let mut num_interfaces = 0u32;
        unsafe {
            if WIN32_ERROR(GetNumberOfInterfaces(&mut num_interfaces)) != NO_ERROR {
                num_interfaces = 16;
            } else {
                num_interfaces = num_interfaces.max(8);
            }
        }
        let mut out_buf_len =
            num_interfaces * std::mem::size_of::<IP_ADAPTER_ADDRESSES_LH>() as u32;
        let mut adapters = Self {
            buf: AdapterAddressBuf::new(out_buf_len as usize)?,
        };
        const MAX_MEMORY_SIZE: u32 = 128 * 1024;
        loop {
            if out_buf_len > MAX_MEMORY_SIZE {
                return Err(io::Error::new(
                    io::ErrorKind::OutOfMemory,
                    "exceeded maximum memory size",
                ));
            }
            match WIN32_ERROR(unsafe {
                GetAdaptersAddresses(
                    0, // AF_UNSPEC
                    GET_ADAPTERS_ADDRESSES_FLAGS(0),
                    None,
                    Some(adapters.buf.ptr),
                    &mut out_buf_len,
                )
            }) {
                NO_ERROR => return Ok(adapters),
                ERROR_BUFFER_OVERFLOW | ERROR_NOT_ENOUGH_MEMORY => {
                    if out_buf_len == MAX_MEMORY_SIZE {
                        return Err(io::Error::new(
                            io::ErrorKind::OutOfMemory,
                            "exceeded maximum memory size",
                        ));
                    }
                    out_buf_len = (out_buf_len * 2).min(MAX_MEMORY_SIZE);
                    adapters.buf = AdapterAddressBuf::new(out_buf_len as usize)?;
                }
                ERROR_NO_DATA => return Err(io::Error::new(io::ErrorKind::NotFound, "No data")),
                other => {
                    return Err(io::Error::other(format!(
                        "GetAdaptersAddresses failed: {:x}",
                        other.0
                    )));
                }
            }
        }
    }
}

fn luid_to_name(luid: NET_LUID_LH) -> String {
    let mut if_index: u32 = 0;
    if unsafe { ConvertInterfaceLuidToIndex(&luid, &mut if_index) } == NO_ERROR {
        let mut buffer = [0u8; IF_NAMESIZE];
        let ptr = unsafe { if_indextoname(if_index, &mut buffer) };
        if !ptr.is_null() {
            if let Ok(name) = unsafe { ptr.to_string() } {
                return name;
            }
        }
    }
    format!("if{:#x}", unsafe { luid.Value })
}

fn sockaddr_to_ipaddr(sock_addr: *const SOCKADDR) -> io::Result<IpAddr> {
    if sock_addr.is_null() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Null pointer"));
    }
    match unsafe { (*sock_addr).sa_family } {
        AF_INET => {
            let sa4 = sock_addr as *const SOCKADDR_IN;
            Ok(IpAddr::V4(
                unsafe { (*sa4).sin_addr.S_un.S_addr }.to_ne_bytes().into(),
            ))
        }
        AF_INET6 => {
            let sa6 = sock_addr as *const SOCKADDR_IN6;
            Ok(IpAddr::V6(unsafe { (*sa6).sin6_addr.u.Byte }.into()))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid address family",
        )),
    }
}
