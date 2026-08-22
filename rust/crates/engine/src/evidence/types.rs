use serde::{Deserialize, Serialize};

mod confidence;
pub use confidence::Confidence;
mod value;
pub use value::{Evidence, EvidenceField, EvidenceValue};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum EvidenceValidator {
    StrongEtag(String),
    LastModified(String),
}

impl EvidenceValidator {
    pub fn strong_etag(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (value.starts_with('"')
            && value.ends_with('"')
            && !value.starts_with("W/")
            && !value.bytes().any(|byte| byte.is_ascii_control()))
        .then_some(Self::StrongEtag(value))
    }

    pub fn last_modified(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty() && !value.bytes().any(|byte| byte.is_ascii_control()))
            .then_some(Self::LastModified(value))
    }

    pub fn is_strong(&self) -> bool {
        matches!(self, Self::StrongEtag(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvidenceScope {
    Event(String),
    EventUrl {
        event: String,
        url: String,
    },
    Url(String),
    ValidatedUrl {
        url: String,
        validator: EvidenceValidator,
    },
    ImmutableBytes(String),
    ClientVersion(String),
}

impl EvidenceScope {
    pub fn url(value: impl Into<String>) -> Self {
        Self::Url(value.into())
    }

    pub fn validated(url: impl Into<String>, validator: EvidenceValidator) -> Self {
        Self::ValidatedUrl {
            url: url.into(),
            validator,
        }
    }

    pub fn event_url(event: impl Into<String>, url: impl Into<String>) -> Self {
        Self::EventUrl {
            event: event.into(),
            url: url.into(),
        }
    }

    pub(crate) fn url_value(&self) -> Option<&str> {
        match self {
            Self::Url(url) | Self::EventUrl { url, .. } | Self::ValidatedUrl { url, .. } => {
                Some(url)
            }
            Self::Event(_) | Self::ImmutableBytes(_) | Self::ClientVersion(_) => None,
        }
    }

    pub(crate) fn validator(&self) -> Option<&EvidenceValidator> {
        match self {
            Self::ValidatedUrl { validator, .. } => Some(validator),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvidenceSource {
    Nostr {
        issuer: String,
        client: Option<String>,
    },
    UrlExtension,
    Head {
        origin: String,
    },
    Response {
        origin: String,
    },
    CompleteBytes {
        origin: String,
    },
    Parser {
        profile: String,
    },
    Hash {
        origin: String,
    },
    Playback {
        client: String,
    },
}

impl EvidenceSource {
    pub fn nostr(issuer: impl Into<String>) -> Self {
        Self::Nostr {
            issuer: issuer.into(),
            client: None,
        }
    }
    pub fn nostr_with_client(issuer: impl Into<String>, client: Option<String>) -> Self {
        Self::Nostr {
            issuer: issuer.into(),
            client,
        }
    }
    pub fn head(origin: impl Into<String>) -> Self {
        Self::Head {
            origin: origin.into(),
        }
    }
    pub fn response(origin: impl Into<String>) -> Self {
        Self::Response {
            origin: origin.into(),
        }
    }
    pub fn parser(profile: impl Into<String>) -> Self {
        Self::Parser {
            profile: profile.into(),
        }
    }
    pub fn hash(origin: impl Into<String>) -> Self {
        Self::Hash {
            origin: origin.into(),
        }
    }
    pub fn playback(client: impl Into<String>) -> Self {
        Self::Playback {
            client: client.into(),
        }
    }

    pub(crate) fn priority(&self) -> u8 {
        match self {
            Self::UrlExtension => 0,
            Self::Nostr { .. } => 1,
            Self::Head { .. } => 2,
            Self::Response { .. } => 3,
            Self::CompleteBytes { .. } => 4,
            Self::Parser { .. } => 5,
            Self::Hash { .. } | Self::Playback { .. } => 6,
        }
    }

    pub(crate) fn direct_bytes(&self) -> bool {
        matches!(
            self,
            Self::Response { .. } | Self::CompleteBytes { .. } | Self::Hash { .. }
        )
    }

    pub(crate) fn structural(&self) -> bool {
        matches!(
            self,
            Self::Parser { .. } | Self::Hash { .. } | Self::Playback { .. }
        )
    }
}
