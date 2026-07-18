use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, de};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ipv4Matcher {
    pub ip: Ipv4Addr,
    pub bits: i32,
}

impl Ipv4Matcher {
    pub const fn matches(&self, addr: Ipv4Addr) -> bool {
        let addr_u32 = u32::from_be_bytes(addr.octets());
        let filter_u32 = u32::from_be_bytes(self.ip.octets());
        let is_suffix = self.bits < 0;
        let bits_abs = self.bits.unsigned_abs();
        let mask = if bits_abs == 0 {
            0
        } else if bits_abs >= 32 {
            u32::MAX
        } else if is_suffix {
            u32::MAX >> (32 - bits_abs)
        } else {
            u32::MAX << (32 - bits_abs)
        };
        (addr_u32 & mask) == (filter_u32 & mask)
    }
}

impl FromStr for Ipv4Matcher {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip_str, bits_str) = s
            .split_once('/')
            .ok_or_else(|| anyhow::anyhow!("invalid matcher format: missing '/'"))?;
        let ip: Ipv4Addr = ip_str
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid IPv4 address in matcher: {e}"))?;

        let bits: i32 = bits_str
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid matcher bit length: {e}"))?;

        anyhow::ensure!(
            bits.unsigned_abs() <= 32,
            "IPv4 matcher bit length cannot exceed 32"
        );

        Ok(Self { ip, bits })
    }
}

impl fmt::Display for Ipv4Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.ip, self.bits)
    }
}

impl<'de> Deserialize<'de> for Ipv4Matcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ipv6Matcher {
    pub ip: Ipv6Addr,
    pub bits: i32,
}

impl Ipv6Matcher {
    pub const fn matches(&self, addr: &Ipv6Addr) -> bool {
        let addr_u128 = u128::from_be_bytes(addr.octets());
        let filter_u128 = u128::from_be_bytes(self.ip.octets());
        let is_suffix = self.bits < 0;
        let bits_abs = self.bits.unsigned_abs();
        let mask = if bits_abs == 0 {
            0
        } else if bits_abs >= 128 {
            u128::MAX
        } else if is_suffix {
            u128::MAX >> (128 - bits_abs)
        } else {
            u128::MAX << (128 - bits_abs)
        };
        (addr_u128 & mask) == (filter_u128 & mask)
    }
}

impl FromStr for Ipv6Matcher {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip_str, bits_str) = s
            .split_once('/')
            .ok_or_else(|| anyhow::anyhow!("invalid matcher format: missing '/'"))?;
        let ip: Ipv6Addr = ip_str
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid IPv6 address in matcher: {e}"))?;

        let bits: i32 = bits_str
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid matcher bit length: {e}"))?;

        anyhow::ensure!(
            bits.unsigned_abs() <= 128,
            "IPv6 matcher bit length cannot exceed 128"
        );

        Ok(Self { ip, bits })
    }
}

impl fmt::Display for Ipv6Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.ip, self.bits)
    }
}

impl<'de> Deserialize<'de> for Ipv6Matcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_filter_v4() -> Result<(), Box<dyn std::error::Error>> {
        let matcher: Ipv4Matcher = "192.168.1.0/24".parse()?;
        assert!(matcher.matches("192.168.1.5".parse()?));
        assert!(matcher.matches("192.168.1.200".parse()?));
        assert!(!matcher.matches("192.168.2.1".parse()?));
        Ok(())
    }

    #[test]
    fn test_prefix_filter_v6() -> Result<(), Box<dyn std::error::Error>> {
        let matcher: Ipv6Matcher = "2001:db8::/64".parse()?;
        assert!(matcher.matches(&"2001:db8::1".parse()?));
        assert!(matcher.matches(&"2001:db8::20".parse()?));
        assert!(!matcher.matches(&"2001:db9::1".parse()?));
        Ok(())
    }

    #[test]
    fn test_suffix_filter_v4() -> Result<(), Box<dyn std::error::Error>> {
        let matcher: Ipv4Matcher = "0.0.0.20/-8".parse()?;
        assert!(matcher.matches("192.168.1.20".parse()?));
        assert!(matcher.matches("10.0.0.20".parse()?));
        assert!(!matcher.matches("192.168.1.21".parse()?));
        Ok(())
    }

    #[test]
    fn test_suffix_filter_v6() -> Result<(), Box<dyn std::error::Error>> {
        let matcher: Ipv6Matcher = "::20/-64".parse()?;
        assert!(matcher.matches(&"2001:db8::20".parse()?));
        assert!(matcher.matches(&"2001:db9::20".parse()?));
        assert!(!matcher.matches(&"2001:db8::21".parse()?));
        Ok(())
    }
}
