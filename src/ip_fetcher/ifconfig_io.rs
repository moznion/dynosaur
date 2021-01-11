use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use thiserror::Error;

use crate::http_client::HttpClient;
use crate::ip_fetcher::ifconfig_io::IfconfigIoIpFetcherError::MalformedIpAddress;
use crate::ip_fetcher::IpFetcher;

const IFCONFIG_IO_URL: &str = "https://ifconfig.io/ip";

#[derive(Error, Debug)]
pub enum IfconfigIoIpFetcherError {
    #[error("malformed ip address: {0}")]
    MalformedIpAddress(String),
}

pub struct IfconfigIo {
    http_client: HttpClient,
}

impl IfconfigIo {
    pub fn new(timeout: Option<Duration>, user_agent: Option<&str>) -> Result<Self> {
        let http_client = HttpClient::new(timeout, user_agent)?;
        Ok(Self { http_client })
    }
}

#[async_trait]
impl IpFetcher for IfconfigIo {
    async fn fetch_public_ip_address(&self) -> Result<IpAddr> {
        let ip_str = self
            .http_client
            .get(IFCONFIG_IO_URL)
            .send()
            .await
            .context("failed to do GET request to ifconfig.io")?
            .text()
            .await?;

        match IpAddr::from_str(ip_str.trim_end()) {
            Ok(ip) => Ok(ip),
            Err(_) => Err(MalformedIpAddress(ip_str.to_owned()))?,
        }
    }
}
