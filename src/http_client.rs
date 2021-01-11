use std::time::Duration;

use anyhow::Result;
use reqwest::{IntoUrl, RequestBuilder};

const DEFAULT_USER_AGENT: &str = "dynosaurd"; // TODO embed the version

pub(crate) struct HttpClient {
    c: reqwest::Client,
}

impl HttpClient {
    pub(crate) fn new(timeout: Option<Duration>, user_agent: Option<&str>) -> Result<Self> {
        let mut client_builder = reqwest::Client::builder();

        if let Some(t) = timeout {
            client_builder = client_builder.timeout(t);
        }

        client_builder = match user_agent {
            Some(ua) => client_builder.user_agent(ua),
            None => client_builder.user_agent(DEFAULT_USER_AGENT),
        };

        let http_client = client_builder.build()?;
        Ok(Self { c: http_client })
    }

    pub(crate) fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.c.get(url)
    }
}
