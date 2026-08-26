use crate::native_cache_failure::permanent;
use crate::public_dns_resolver::{PublicDnsResolver, SystemResolver};
use crate::public_media_address::validate_url;
use anyhow::{Context as _, Result};
use core::time::Duration;
use reqwest::dns::Resolve;
use reqwest::redirect::Policy;
use reqwest::{Client, ClientBuilder, RequestBuilder, Url};
use std::sync::Arc;

pub const MEDIA_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const MEDIA_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone, Copy)]
pub struct MediaHttpTimeouts {
    connect: Duration,
    request: Duration,
}

impl MediaHttpTimeouts {
    fn production() -> Self {
        Self {
            connect: MEDIA_CONNECT_TIMEOUT,
            request: MEDIA_REQUEST_TIMEOUT,
        }
    }
}

#[derive(Clone)]
pub struct MediaHttpClient {
    client: Client,
}

/// Request-only port for media consumers that must not depend on the
/// concrete guarded HTTP client.
pub trait MediaHttpRequests: Send + Sync {
    /// Builds one guarded GET hop for `raw_url`; automatic redirects must be disabled.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is unsafe or a request cannot be built.
    fn get(&self, raw_url: &str) -> Result<RequestBuilder>;
}

impl MediaHttpClient {
    /// # Errors
    ///
    /// Returns an error when the guarded production client cannot be constructed.
    pub fn public() -> Result<Self> {
        Self::with_resolver(Arc::new(SystemResolver))
    }

    /// # Errors
    ///
    /// Returns an error when a guarded client cannot be constructed with `resolver`.
    pub fn with_resolver<R: Resolve + 'static>(resolver: Arc<R>) -> Result<Self> {
        let resolver = Arc::new(PublicDnsResolver::new(resolver));
        let client = media_client_builder(MediaHttpTimeouts::production())
            .dns_resolver(resolver)
            .build()
            .context("build media HTTP client")?;
        Ok(Self { client })
    }

    /// # Errors
    ///
    /// Returns an error when `raw_url` is unsafe or a request cannot be built.
    pub fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        validate_initial_url(raw_url)?;
        Ok(self.client.get(raw_url))
    }
}

impl MediaHttpRequests for MediaHttpClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        Self::get(self, raw_url)
    }
}

impl<T: MediaHttpRequests + ?Sized> MediaHttpRequests for Arc<T> {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        self.as_ref().get(raw_url)
    }
}

fn media_client_builder(timeouts: MediaHttpTimeouts) -> ClientBuilder {
    Client::builder()
        .no_proxy()
        .connect_timeout(timeouts.connect)
        .timeout(timeouts.request)
        .redirect(Policy::none())
}

fn validate_initial_url(raw_url: &str) -> Result<()> {
    let url = Url::parse(raw_url).map_err(|_parse_error| permanent("media URL is invalid"))?;
    validate_url(&url)
}
