use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Default)]
pub struct LookupCache {
    v4: Option<Ipv4Addr>,
    v6: Option<Ipv6Addr>,
}

pub enum UpdateResult<T> {
    Initialized,
    Updated(T),
    Unchanged,
}

impl LookupCache {
    pub fn update_v4(&mut self, v4: &Ipv4Addr) -> UpdateResult<Ipv4Addr> {
        if let Some(prev) = self.v4
            && &prev == v4
        {
            return UpdateResult::Unchanged;
        }
        match self.v4.replace(*v4) {
            None => UpdateResult::Initialized,
            Some(old) => UpdateResult::Updated(old),
        }
    }

    pub fn update_v6(&mut self, v6: &Ipv6Addr) -> UpdateResult<Ipv6Addr> {
        if let Some(prev) = self.v6
            && &prev == v6
        {
            return UpdateResult::Unchanged;
        }
        match self.v6.replace(*v6) {
            None => UpdateResult::Initialized,
            Some(old) => UpdateResult::Updated(old),
        }
    }
}
