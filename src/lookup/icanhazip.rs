use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::lookup::Lookup;

pub struct ICanHazIp {
    client: Client,
}

impl ICanHazIp {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self { client })
    }
}

impl ICanHazIp {
    async fn lookup<T: FromStr<Err = AddrParseError>>(&self, url: &str) -> Result<T> {
        let body = self.client.get(url).send().await?.text().await?;
        body.trim() // ends with \n
            .parse()
            .with_context(|| format!("unable to parse {body}"))
    }
}

impl Lookup for ICanHazIp {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        self.lookup("https://ipv4.icanhazip.com").await
    }
    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        self.lookup("https://ipv6.icanhazip.com").await
    }
}

#[cfg(test)]
mod tests {
    use crate::lookup::Lookup;
    use crate::lookup::icanhazip::ICanHazIp;

    #[tokio::test]
    #[ignore]
    async fn v4_test() {
        let r = ICanHazIp::new().unwrap().lookup_v4().await.unwrap();
        println!("{r:?}")
    }

    #[tokio::test]
    #[ignore]
    async fn v6_test() {
        let r = ICanHazIp::new().unwrap().lookup_v6().await.unwrap();
        println!("{r:?}")
    }
}
