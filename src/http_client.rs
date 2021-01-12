use std::time::Duration;

use anyhow::Result;
use reqwest::{IntoUrl, RequestBuilder};

pub struct HttpClient {
    c: reqwest::Client,
}

impl HttpClient {
    const DEFAULT_USER_AGENT: &'static str = "dynosaur"; // TODO embed the version

    pub fn new(timeout: Option<Duration>, user_agent: Option<&str>) -> Result<Self> {
        let mut client_builder = reqwest::Client::builder();

        if let Some(t) = timeout {
            client_builder = client_builder.timeout(t);
        }

        client_builder = match user_agent {
            Some(ua) => client_builder.user_agent(ua),
            None => client_builder.user_agent(Self::DEFAULT_USER_AGENT),
        };

        let http_client = client_builder.build()?;
        Ok(Self { c: http_client })
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.c.get(url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.c.post(url)
    }

    pub fn put<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.c.put(url)
    }
}
