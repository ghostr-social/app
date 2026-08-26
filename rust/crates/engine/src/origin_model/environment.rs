use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Availability<T> {
    Available(T),
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DomainClass {
    FirstParty,
    ContentDeliveryNetwork,
    ObjectStorage,
    SmallProvider,
    PublicHost,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum HttpProtocol {
    Http1,
    Http2,
    Http3,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum TlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OriginEnvironment {
    pub(super) domain_class: Availability<DomainClass>,
    pub(super) hosting_service: Availability<String>,
    pub(crate) asn: Availability<u32>,
    pub(crate) region: Availability<String>,
    pub(super) protocol: Availability<HttpProtocol>,
    pub(super) tls_version: Availability<TlsVersion>,
}

impl OriginEnvironment {
    pub(crate) fn unavailable() -> Self {
        Self {
            domain_class: Availability::Unavailable,
            hosting_service: Availability::Unavailable,
            asn: Availability::Unavailable,
            region: Availability::Unavailable,
            protocol: Availability::Unavailable,
            tls_version: Availability::Unavailable,
        }
    }
}

#[cfg(test)]
#[path = "environment/test_support.rs"]
mod test_support;

impl Default for OriginEnvironment {
    fn default() -> Self {
        Self::unavailable()
    }
}
