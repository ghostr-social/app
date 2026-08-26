use anyhow::{ensure, Result};
use core::fmt::Write as _;
use core::fmt::{Display, Formatter};
use core::time::Duration;
use rand::RngCore as _;
use reqwest::Url;

const MAX_HLS_SOURCE_URLS: usize = 5;
const MAX_HLS_URL_BYTES: usize = 2_048;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HlsSessionId(pub(crate) String);

impl HlsSessionId {
    pub fn parse(raw: &str) -> Option<Self> {
        (raw.len() == 64 && raw.bytes().all(|value| value.is_ascii_hexdigit()))
            .then(|| Self(raw.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const MAX_RESOURCE_TOKEN_BYTES: usize = 16_384;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HlsResourceId(pub(crate) String);

impl HlsResourceId {
    pub fn parse(raw: &str) -> Option<Self> {
        (!raw.is_empty()
            && raw.len() <= MAX_RESOURCE_TOKEN_BYTES
            && raw
                .bytes()
                .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_')))
        .then(|| Self(raw.to_owned()))
    }
}

impl Display for HlsResourceId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy)]
pub struct HlsSessionLimits {
    pub(super) max_sessions: usize,
    pub(super) idle_ttl: Duration,
    pub(super) max_ranged_assets: usize,
}

impl HlsSessionLimits {
    /// Creates positive capacity and lifetime limits for secure HLS sessions.
    ///
    /// # Errors
    ///
    /// Returns an error when any capacity is zero or the idle lifetime is empty.
    pub fn new(max_sessions: usize, idle_ttl: Duration, max_ranged_assets: usize) -> Result<Self> {
        ensure!(max_sessions > 0, "HLS session capacity must be positive");
        ensure!(!idle_ttl.is_zero(), "HLS session TTL must be positive");
        ensure!(
            max_ranged_assets > 0,
            "HLS ranged asset capacity must be positive"
        );
        Ok(Self {
            max_sessions,
            idle_ttl,
            max_ranged_assets,
        })
    }
}

pub(crate) fn validated_sources(sources: Vec<String>) -> Result<Vec<Url>> {
    ensure!(!sources.is_empty(), "an HLS source is required");
    ensure!(sources.len() <= MAX_HLS_SOURCE_URLS, "too many HLS sources");
    sources.into_iter().map(|raw| validated_url(&raw)).collect()
}

fn validated_url(raw: &str) -> Result<Url> {
    ensure!(raw.len() <= MAX_HLS_URL_BYTES, "HLS source URL is too long");
    let url = Url::parse(raw)?;
    ensure!(
        matches!(url.scheme(), "http" | "https"),
        "HLS source scheme is not allowed"
    );
    ensure!(url.host().is_some(), "HLS source host is required");
    ensure!(
        url.username().is_empty(),
        "HLS source credentials are forbidden"
    );
    ensure!(
        url.password().is_none(),
        "HLS source credentials are forbidden"
    );
    Ok(url)
}

pub(crate) fn random_id() -> HlsSessionId {
    HlsSessionId(hex_secret(random_secret()))
}

pub(crate) fn random_secret() -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

fn hex_secret(bytes: [u8; 32]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for value in bytes {
        write!(&mut encoded, "{value:02x}").expect("writing to a string cannot fail");
    }
    encoded
}
