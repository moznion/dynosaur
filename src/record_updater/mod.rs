use std::net::IpAddr;

use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

pub mod cloudflare;

#[async_trait]
pub trait RecordUpdater {
    async fn update(
        &self,
        ip: IpAddr,
        record_type: &str,
        record_name: &str,
        ttl: Option<Duration>,
    ) -> Result<()>;
}
