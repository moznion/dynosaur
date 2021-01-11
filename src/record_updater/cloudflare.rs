use std::net::IpAddr;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use thiserror::Error;

use serde::{Deserialize, Serialize};

use crate::http_client::HttpClient;
use crate::record_updater::cloudflare::HttpbinIpFetcherError::{
    AmbiguousDNSRecords, DNSRecordRetrievalFailed,
};
use crate::record_updater::RecordUpdater;

#[derive(Error, Debug)]
pub enum HttpbinIpFetcherError {
    #[error(
        "There are two or more bound DNS records belong to {0} of {1} type. This has to be zero or exact one record."
    )]
    AmbiguousDNSRecords(String, String),
    #[error("Failed to retrieve DNS record information; API response indicated success is false explicitly.")]
    DNSRecordRetrievalFailed,
}

pub struct Cloudflare {
    api_token: String,
    zone_id: String,
    http_client: HttpClient,
}

#[derive(Deserialize, Debug)]
struct DNSRecordsResponse {
    result: Vec<DNSRecordsResult>,
    success: bool,
}

#[derive(Deserialize, Debug)]
struct DNSRecordsResult {
    id: String,
    name: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct DNSCreateRequest {
    #[serde(rename = "type")]
    typ: String,
    name: String,
    content: String,
    ttl: u64,
}

impl Cloudflare {
    // NOTE:
    // The reason why not using `cloudflare/cloudflare-rs` is this library is not stable for now,
    // as that described:
    // > ⚠️ This library is a Work in Progress! ⚠️
    // https://github.com/cloudflare/cloudflare-rs/blob/80d249c0cc8241e1043a59c1fb6e23979ce62cae/README.md

    const DEFAULT_RECORD_TTL: u64 = 1;

    pub fn new(
        api_token: &str,
        zone_id: &str,
        timeout: Option<Duration>,
        user_agent: Option<&str>,
    ) -> Result<Self> {
        let http_client = HttpClient::new(timeout, user_agent)?;
        Ok(Self {
            api_token: api_token.to_owned(),
            zone_id: zone_id.to_owned(),
            http_client,
        })
    }

    fn construct_records_api_url(&self) -> String {
        format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        )
    }

    async fn retrieve_records(
        &self,
        record_type: &str,
        record_name: &str,
    ) -> Result<DNSRecordsResponse> {
        let resp: DNSRecordsResponse = self
            .http_client
            .get(&self.construct_records_api_url())
            .query(&[("type", record_type), ("name", record_name)])
            .bearer_auth(&self.api_token)
            .send()
            .await?
            .json()
            .await?;

        if !resp.success {
            return Err(DNSRecordRetrievalFailed)?;
        }
        Ok(resp)
    }

    async fn create_record(
        &self,
        ip: IpAddr,
        record_type: &str,
        record_name: &str,
        ttl: Option<Duration>,
    ) -> Result<()> {
        self.http_client
            .post(&self.construct_records_api_url())
            .bearer_auth(&self.api_token)
            .header("Content-Type", "application/json")
            .json(&Self::construct_dns_create_request(
                ip,
                record_type,
                record_name,
                ttl,
            ))
            .send()
            .await?;
        Ok(())
    }

    async fn update_record(
        &self,
        record_id: &str,
        ip: IpAddr,
        record_type: &str,
        record_name: &str,
        ttl: Option<Duration>,
    ) -> Result<()> {
        self.http_client
            .put(&format!(
                "{}/{}",
                self.construct_records_api_url(),
                record_id
            ))
            .bearer_auth(&self.api_token)
            .header("Content-Type", "application/json")
            .json(&Self::construct_dns_create_request(
                ip,
                record_type,
                record_name,
                ttl,
            ))
            .send()
            .await?;
        Ok(())
    }

    fn construct_dns_create_request(
        ip: IpAddr,
        record_type: &str,
        record_name: &str,
        ttl: Option<Duration>,
    ) -> DNSCreateRequest {
        DNSCreateRequest {
            typ: record_type.to_owned(),
            name: record_name.to_owned(),
            content: ip.to_string(),
            ttl: ttl.map_or(Self::DEFAULT_RECORD_TTL, |t| t.as_secs()),
        }
    }
}

#[async_trait]
impl RecordUpdater for Cloudflare {
    async fn update(
        &self,
        ip: IpAddr,
        record_type: &str,
        record_name: &str,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let resp = self.retrieve_records(record_type, record_name).await?;
        match resp.result.len() {
            0 => self.create_record(ip, record_type, record_name, ttl).await,
            1 => {
                if resp.result[0].content == ip.to_string() {
                    // not changed; there is nothing to do
                    return Ok(());
                }

                self.update_record(&resp.result[0].id, ip, record_type, record_name, ttl)
                    .await
            }
            _ => Err(AmbiguousDNSRecords(
                record_name.to_owned(),
                record_type.to_owned(),
            ))?,
        }
    }
}
