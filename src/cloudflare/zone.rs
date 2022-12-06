//! Partial implementation, only contains fields that we'll use

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::cloudflare::ApiRequest;

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

    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> String {
        "zones".to_string()
    }
}
