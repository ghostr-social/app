use super::*;

impl ColdStartPrior {
    pub(crate) fn new(
        success_alpha: f64,
        success_beta: f64,
        ttfb_ms: u64,
        throughput_bps: u64,
    ) -> Self {
        Self {
            success_alpha: success_alpha.max(0.1),
            success_beta: success_beta.max(0.1),
            range_alpha: 3.0,
            range_beta: 2.0,
            ttfb_p50_ms: ttfb_ms.max(1),
            throughput_p50_bps: throughput_bps.max(1),
        }
    }
}

impl ColdStartSelector {
    pub(crate) fn with_domain_class(mut self, value: DomainClass) -> Self {
        self.domain_class = Some(value);
        self
    }

    pub(crate) fn with_protocol(mut self, value: HttpProtocol) -> Self {
        self.protocol = Some(value);
        self
    }

    pub(crate) fn with_tls_version(mut self, value: TlsVersion) -> Self {
        self.tls_version = Some(value);
        self
    }

    pub(crate) fn with_method(mut self, value: RequestMethod) -> Self {
        self.method = Some(value);
        self
    }
}
