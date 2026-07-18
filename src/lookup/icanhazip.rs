use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::lookup::LookupSpec;

pub struct ICanHazIp {
    client: Client,
}

impl ICanHazIp {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .no_proxy()
            .timeout(Duration::from_mins(1))
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

impl LookupSpec for ICanHazIp {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        self.lookup("https://ipv4.icanhazip.com").await
    }
    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        self.lookup("https://ipv6.icanhazip.com").await
    }
}

#[cfg(test)]
#[expect(clippy::print_stdout, reason = "print_stdout allowed in tests")]
mod tests {
    use crate::lookup::LookupSpec;
    use crate::lookup::icanhazip::ICanHazIp;

    #[tokio::test]
    #[ignore = "requires public network"]
    async fn v4_test() -> anyhow::Result<()> {
        let r = ICanHazIp::new()?.lookup_v4().await?;
        println!("{r:?}");
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires public network"]
    async fn v6_test() -> anyhow::Result<()> {
        let r = ICanHazIp::new()?.lookup_v6().await?;
        println!("{r:?}");
        Ok(())
    }
}
