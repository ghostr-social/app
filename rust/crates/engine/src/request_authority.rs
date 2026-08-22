use serde::{Deserialize, Serialize};
use url::Url;

/// Canonical connection-pool identity for one credential-free HTTP origin.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RequestAuthority(String);

impl RequestAuthority {
    pub fn from_url(source: &str) -> Option<Self> {
        let url = Url::parse(source).ok()?;
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.host().is_none()
        {
            return None;
        }
        Some(Self(url.origin().ascii_serialization()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
