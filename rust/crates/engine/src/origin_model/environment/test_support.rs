use super::*;

impl OriginEnvironment {
    pub(crate) fn with_domain_class(mut self, value: DomainClass) -> Self {
        self.domain_class = Availability::Available(value);
        self
    }

    pub(crate) fn with_protocol(mut self, value: HttpProtocol) -> Self {
        self.protocol = Availability::Available(value);
        self
    }

    pub(crate) fn with_tls_version(mut self, value: TlsVersion) -> Self {
        self.tls_version = Availability::Available(value);
        self
    }
}
