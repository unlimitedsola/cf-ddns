use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::lookup::LookupProvider;

pub struct ICanHazIp;

#[async_trait]
impl LookupProvider for ICanHazIp {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        lookup("https://ipv4.icanhazip.com").await
    }
    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        lookup("https://ipv6.icanhazip.com").await
    }
}

async fn lookup<T: FromStr<Err = AddrParseError>>(url: &str) -> Result<T> {
    let body = reqwest::get(url).await?.text().await?;
    body.trim() // ends with \n
        .parse()
        .with_context(|| format!("unable to parse {body}"))
}

#[cfg(test)]
mod tests {
    use crate::lookup::icanhazip::ICanHazIp;
    use crate::lookup::LookupProvider;

    #[tokio::test]
    async fn v4_test() {
        let r = ICanHazIp.lookup_v4().await.unwrap();
        println!("{r:?}")
    }

    #[tokio::test]
    async fn v6_test() {
        let r = ICanHazIp.lookup_v6().await.unwrap();
        println!("{r:?}")
    }
}
