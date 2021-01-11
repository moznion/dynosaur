use std::net::IpAddr;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait IpFetcher {
    async fn fetch_public_ip_address(&self) -> Result<IpAddr>;
}

pub mod httpbin;
pub mod ifconfig_io;
