//! Debug-build media client for deterministic on-device integration origins.

use anyhow::{Context, Result};
use ghostr_net::outbound_media_client::{
    MediaHttpClient, MediaHttpRequests, MEDIA_CONNECT_TIMEOUT, MEDIA_REQUEST_TIMEOUT,
};
use reqwest::redirect::Policy;
use reqwest::{RequestBuilder, Url};

#[derive(Clone)]
pub struct DeviceIntegrationMediaHttpClient {
    public: MediaHttpClient,
    loopback: reqwest::Client,
    allowed_origin: Url,
}

impl DeviceIntegrationMediaHttpClient {
    pub fn new(raw_origin: &str) -> Result<Self> {
        let allowed_origin = allowed_origin(raw_origin)?;
        let loopback = reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(MEDIA_CONNECT_TIMEOUT)
            .timeout(MEDIA_REQUEST_TIMEOUT)
            .redirect(Policy::none())
            .build()
            .context("build device integration media client")?;
        Ok(Self {
            public: MediaHttpClient::public()?,
            loopback,
            allowed_origin,
        })
    }
}

impl MediaHttpRequests for DeviceIntegrationMediaHttpClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        if matches_origin(raw_url, &self.allowed_origin) {
            return Ok(self.loopback.get(raw_url));
        }
        self.public.get(raw_url)
    }
}

fn allowed_origin(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).context("parse device integration origin")?;
    anyhow::ensure!(url.scheme() == "http", "integration origin must use HTTP");
    anyhow::ensure!(
        url.username().is_empty(),
        "integration origin has credentials"
    );
    anyhow::ensure!(
        url.password().is_none(),
        "integration origin has credentials"
    );
    anyhow::ensure!(url.path() == "/", "integration origin must not have a path");
    anyhow::ensure!(
        url.query().is_none(),
        "integration origin must not have a query"
    );
    anyhow::ensure!(
        url.fragment().is_none(),
        "integration origin must not have a fragment"
    );
    anyhow::ensure!(url.port().is_some(), "integration origin must have a port");
    let host = url.host_str().context("integration origin has no host")?;
    let ip: std::net::IpAddr = host
        .parse()
        .context("integration origin is not literal IP")?;
    anyhow::ensure!(ip.is_loopback(), "integration origin is not loopback");
    Ok(url)
}

fn matches_origin(raw: &str, allowed: &Url) -> bool {
    Url::parse(raw).is_ok_and(|url| {
        url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none()
            && url.origin() == allowed.origin()
    })
}
