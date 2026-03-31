use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::config::ProviderConfig;

#[derive(Debug, Default)]
pub struct LookupCache {
    v4: HashMap<ProviderConfig, Ipv4Addr>,
    v6: HashMap<ProviderConfig, Ipv6Addr>,
}

pub enum UpdateResult<T> {
    Initialized,
    Updated(T),
    Unchanged,
}

impl LookupCache {
    pub fn update_v4(&mut self, key: &ProviderConfig, v4: Ipv4Addr) -> UpdateResult<Ipv4Addr> {
        if let Some(entry) = self.v4.get_mut(key) {
            if *entry == v4 {
                return UpdateResult::Unchanged;
            }
            let old = std::mem::replace(entry, v4);
            return UpdateResult::Updated(old);
        }
        self.v4.insert(key.clone(), v4);
        UpdateResult::Initialized
    }

    pub fn update_v6(&mut self, key: &ProviderConfig, v6: Ipv6Addr) -> UpdateResult<Ipv6Addr> {
        if let Some(entry) = self.v6.get_mut(key) {
            if *entry == v6 {
                return UpdateResult::Unchanged;
            }
            let old = std::mem::replace(entry, v6);
            return UpdateResult::Updated(old);
        }
        self.v6.insert(key.clone(), v6);
        UpdateResult::Initialized
    }
}
