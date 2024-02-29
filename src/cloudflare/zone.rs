//! Partial implementation, only contains fields that we'll use

use const_format::concatcp;
use serde::{Deserialize, Serialize};

use crate::cloudflare::{ApiRequest, BASE_URL};

#[derive(Deserialize, Debug)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

/// List Zones
/// https://api.cloudflare.com/#zone-list-zones
#[derive(Serialize, Debug)]
pub struct ListZones;

impl ApiRequest for ListZones {
    type Request = ();
    type Query = ();
    type Response = Vec<Zone>;

    fn url(&self) -> &str {
        concatcp!(BASE_URL, "/zones")
    }
}
