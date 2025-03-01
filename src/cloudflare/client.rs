//! Client exchange implementation for CloudFlare API

use anyhow::Result;
use anyhow::bail;
use reqwest::{IntoUrl, Method};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::cloudflare::CloudFlare;

pub trait ApiRequest {
    type Request: Serialize;
    type Query: Serialize;
    type Response: DeserializeOwned;

    fn method(&self) -> Method {
        Method::GET
    }
    fn url(&self) -> impl IntoUrl;
    fn query(&self) -> Option<&Self::Query> {
        None
    }
    fn body(&self) -> Option<&Self::Request> {
        None
    }
}

pub const BASE_URL: &str = "https://api.cloudflare.com/client/v4"; // no trailing slash

// Base exchange implementation
impl CloudFlare {
    pub async fn call<Api>(&self, api: &Api) -> Result<Api::Response>
    where
        Api: ApiRequest,
    {
        let mut request = self
            .http
            .request(api.method(), api.url())
            .query(&api.query());

        if let Some(body) = api.body() {
            request = request.json(body);
        }
        Response::extract(request.send().await?).await
    }
}

#[derive(Deserialize, Debug)]
struct Response<T> {
    pub result: T,
}

impl<T> Response<T> {
    pub async fn extract(resp: reqwest::Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = resp.status();
        if status.is_success() {
            Ok(resp.json::<Response<T>>().await?.result)
        } else {
            bail!(
                "Error from Cloudflare API. status: {}, response: {}",
                status,
                resp.text().await?,
            )
        }
    }
}
