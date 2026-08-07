use crate::native_cache_failure::{permanent, permanent_cause};
use crate::public_dns_resolver::{PublicDnsResolver, SystemResolver};
use crate::public_media_address::validate_url;
use anyhow::{ensure, Context, Result};
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
    pub fn new(connect: Duration, request: Duration) -> Result<Self> {
        ensure!(!connect.is_zero(), "media connect timeout must be positive");
        ensure!(!request.is_zero(), "media request timeout must be positive");
        Ok(Self { connect, request })
    }

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
    enforce_public_destinations: bool,
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
        Ok(Self {
            client,
            enforce_public_destinations: true,
        })
    }

    pub fn trusted() -> Result<Self> {
        Self::trusted_with_timeouts(MediaHttpTimeouts::production())
    }

    pub fn trusted_with_timeouts(timeouts: MediaHttpTimeouts) -> Result<Self> {
        let client = media_client_builder(timeouts)
            .build()
            .context("build trusted media HTTP client")?;
        Ok(Self {
            client,
            enforce_public_destinations: false,
        })
    }

    pub fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        if self.enforce_public_destinations {
            validate_initial_url(raw_url)?;
        }
        Ok(self.client.get(raw_url))
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

fn public_redirect_policy() -> Policy {
    Policy::custom(redirect_action)
}

fn redirect_action(attempt: Attempt<'_>) -> reqwest::redirect::Action {
    if attempt.previous().len() > 10 {
        return attempt.error(permanent_cause("too many media redirects"));
    }
    if validate_url(attempt.url()).is_err() {
        return attempt.error(permanent_cause("media redirect target is not public"));
    }
    attempt.follow()
}
