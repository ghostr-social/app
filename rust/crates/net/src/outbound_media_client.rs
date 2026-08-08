use crate::native_cache_failure::{permanent, permanent_cause};
use crate::public_dns_resolver::{PublicDnsResolver, SystemResolver};
use crate::public_media_address::validate_url;
use anyhow::{Context, Result};
use reqwest::dns::Resolve;
use reqwest::redirect::{Attempt, Policy};
use reqwest::{Client, ClientBuilder, RequestBuilder, Url};
use std::sync::Arc;
use std::time::Duration;

const MEDIA_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MEDIA_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

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
    /// Starts a guarded GET request for `raw_url`.
    fn get(&self, raw_url: &str) -> Result<RequestBuilder>;
}

impl MediaHttpClient {
    pub fn public() -> Result<Self> {
        Self::with_resolver(Arc::new(SystemResolver))
    }

    pub fn with_resolver<R: Resolve + 'static>(resolver: Arc<R>) -> Result<Self> {
        let resolver = Arc::new(PublicDnsResolver::new(resolver));
        let client = media_client_builder(MediaHttpTimeouts::production())
            .dns_resolver(resolver)
            .build()
            .context("build media HTTP client")?;
        Ok(Self { client })
    }

    pub fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        validate_initial_url(raw_url)?;
        Ok(self.client.get(raw_url))
    }
}

impl MediaHttpRequests for MediaHttpClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        MediaHttpClient::get(self, raw_url)
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
        .redirect(public_redirect_policy())
}

fn validate_initial_url(raw_url: &str) -> Result<()> {
    let url = Url::parse(raw_url).map_err(|_| permanent("media URL is invalid"))?;
    validate_url(&url)
}

pub(crate) fn public_redirect_policy() -> Policy {
    Policy::custom(redirect_action)
}

fn redirect_action(attempt: Attempt<'_>) -> reqwest::redirect::Action {
    if validate_url(attempt.url()).is_err() {
        return attempt.error(permanent_cause("media redirect target is not public"));
    }
    Policy::limited(10).redirect(attempt)
}
