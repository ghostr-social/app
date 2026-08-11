//! Explicit media access for deterministic, loopback-only debug fixtures.

use anyhow::{Context, Result};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use reqwest::redirect::Policy;
use reqwest::{RequestBuilder, Url};

#[derive(Clone)]
pub struct DebugMediaHttpClient {
    public: MediaHttpClient,
    loopback: reqwest::Client,
}

impl DebugMediaHttpClient {
    pub fn new() -> Result<Self> {
        let loopback = reqwest::Client::builder()
            .no_proxy()
            .redirect(Policy::none())
            .build()
            .context("build debug loopback media client")?;
        Ok(Self {
            public: MediaHttpClient::public()?,
            loopback,
        })
    }
}

impl MediaHttpRequests for DebugMediaHttpClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        if is_literal_loopback(raw_url) {
            return Ok(self.loopback.get(raw_url));
        }
        self.public.get(raw_url)
    }
}

fn is_literal_loopback(raw_url: &str) -> bool {
    let Ok(url) = Url::parse(raw_url) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https")
        && url.host().is_some_and(|host| {
            host.to_string()
                .parse()
                .is_ok_and(|ip: std::net::IpAddr| ip.is_loopback())
        })
}
