//! Partial implementation, only contains field that we'll use
use crate::cloudflare::ApiRequest;
use reqwest::Method;
use serde::{Deserialize, Serialize};

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
