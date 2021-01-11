use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use thiserror::Error;

use crate::http_client::HttpClient;
use crate::ip_fetcher::httpbin::HttpbinIpFetcherError::{
    MalformedIpAddress, MissingOriginProperty,
};
use crate::ip_fetcher::IpFetcher;

#[derive(Error, Debug)]
pub enum HttpbinIpFetcherError {
    #[error("missing `origin` property in a response of GET httpbin ip")]
    MissingOriginProperty,
    #[error("malformed ip address: {0}")]
    MalformedIpAddress(String),
}

const HTTPBIN_IP_URL: &str = "https://httpbin.org/ip";
const ORIGIN_PROPERTY_KEY: &str = "origin";

pub struct Httpbin {
    http_client: HttpClient,
}

impl Httpbin {
    pub fn new(timeout: Option<Duration>, user_agent: Option<&str>) -> Result<Self> {
        let http_client = HttpClient::new(timeout, user_agent)?;
        Ok(Self { http_client })
    }
}

#[async_trait]
impl IpFetcher for Httpbin {
    async fn fetch_public_ip_address(&self) -> Result<IpAddr> {
        let resp = self
            .http_client
            .get(HTTPBIN_IP_URL)
            .send()
            .await
            .context("failed to do GET request to httpbin")?
            .json::<HashMap<String, String>>()
            .await
            .context("failed to map the response to a JSON")?;

        match resp.get(ORIGIN_PROPERTY_KEY) {
            Some(ip_str) => match IpAddr::from_str(ip_str) {
                Ok(ip) => Ok(ip),
                Err(_) => Err(MalformedIpAddress(ip_str.to_owned()))?,
            },
            None => Err(MissingOriginProperty)?,
        }
    }
}
