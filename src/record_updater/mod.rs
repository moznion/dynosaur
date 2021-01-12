use std::net::IpAddr;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;

pub mod cloudflare;

#[async_trait]
pub trait RecordUpdater {
    async fn update(&self, ip: &IpAddr, subject_record: &SubjectRecord) -> Result<()>;
}

pub struct SubjectRecord {
    record_type: String,
    record_name: String,
    ttl: Option<Duration>,
}

impl SubjectRecord {
    pub fn new(record_type: &str, record_name: &str, ttl: Option<Duration>) -> Self {
        Self {
            record_type: String::from(record_type),
            record_name: String::from(record_name),
            ttl,
        }
    }

    pub fn get_record_type(&self) -> &String {
        &self.record_type
    }

    pub fn get_record_name(&self) -> &String {
        &self.record_name
    }
}
