use core::fmt::{Display, Formatter, Write as _};
use reqwest::Url;
use sha2::{Digest as _, Sha256};

/// Correlatable media identity for logs that never exposes a source URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaLogIdentity {
    origin: String,
    object: String,
}

impl MediaLogIdentity {
    pub fn from_url(value: &str) -> Self {
        let parsed = Url::parse(value).ok();
        let origin = parsed.as_ref().map_or_else(
            || "invalid".to_owned(),
            |url| url.origin().ascii_serialization(),
        );
        let object = parsed.as_ref().map_or(value, Url::as_str);
        Self {
            origin: digest(origin.as_bytes()),
            object: digest(object.as_bytes()),
        }
    }
}

impl Display for MediaLogIdentity {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "media(origin={}, object={})",
            self.origin, self.object
        )
    }
}

fn digest(value: &[u8]) -> String {
    let bytes = Sha256::digest(value);
    let mut encoded = String::with_capacity(16);
    for byte in &bytes[..8] {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}
