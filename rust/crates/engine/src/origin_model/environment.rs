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
    pub domain_class: Availability<DomainClass>,
    pub hosting_service: Availability<String>,
    pub asn: Availability<u32>,
    pub region: Availability<String>,
    pub protocol: Availability<HttpProtocol>,
    pub tls_version: Availability<TlsVersion>,
}

impl OriginEnvironment {
    pub fn unavailable() -> Self {
        Self {
            domain_class: Availability::Unavailable,
            hosting_service: Availability::Unavailable,
            asn: Availability::Unavailable,
            region: Availability::Unavailable,
            protocol: Availability::Unavailable,
            tls_version: Availability::Unavailable,
        }
    }

    pub fn with_domain_class(mut self, value: DomainClass) -> Self {
        self.domain_class = Availability::Available(value);
        self
    }

    pub fn with_hosting_service(mut self, value: impl Into<String>) -> Self {
        self.hosting_service = Availability::Available(value.into());
        self
    }

    pub fn with_asn(mut self, value: u32) -> Self {
        self.asn = Availability::Available(value);
        self
    }

    pub fn with_region(mut self, value: impl Into<String>) -> Self {
        self.region = Availability::Available(value.into());
        self
    }

    pub fn with_protocol(mut self, value: HttpProtocol) -> Self {
        self.protocol = Availability::Available(value);
        self
    }

    pub fn with_tls_version(mut self, value: TlsVersion) -> Self {
        self.tls_version = Availability::Available(value);
        self
    }
}

impl Default for OriginEnvironment {
    fn default() -> Self {
        Self::unavailable()
    }
}
