use super::{Availability, DomainClass, HttpProtocol, OriginQuery, RequestMethod, TlsVersion};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ColdStartPrior {
    pub(super) success_alpha: f64,
    pub(super) success_beta: f64,
    pub(crate) range_alpha: f64,
    pub(crate) range_beta: f64,
    pub(super) ttfb_p50_ms: u64,
    pub(super) throughput_p50_bps: u64,
}

impl ColdStartPrior {
    pub(super) fn bootstrap(method: RequestMethod) -> Self {
        let (success, range, ttfb, throughput) = match method {
            RequestMethod::Head | RequestMethod::ManifestGet | RequestMethod::SegmentGet => {
                ((8.0, 2.0), (1.0, 1.0), 250, 4_194_304)
            }
            RequestMethod::RangeGet | RequestMethod::PrefixGet | RequestMethod::TailGet => {
                ((7.0, 3.0), (3.0, 2.0), 300, 3_145_728)
            }
            RequestMethod::FullGet => ((7.0, 3.0), (1.0, 1.0), 300, 3_145_728),
        };
        Self {
            success_alpha: success.0,
            success_beta: success.1,
            range_alpha: range.0,
            range_beta: range.1,
            ttfb_p50_ms: ttfb,
            throughput_p50_bps: throughput,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ColdStartSelector {
    domain_class: Option<DomainClass>,
    hosting_service: Option<String>,
    asn: Option<u32>,
    region: Option<String>,
    protocol: Option<HttpProtocol>,
    #[serde(default)]
    tls_version: Option<TlsVersion>,
    method: Option<RequestMethod>,
}

impl ColdStartSelector {
    pub(super) fn score(&self, query: &OriginQuery) -> Option<usize> {
        let env = &query.environment;
        let scores = [
            match_value(self.domain_class.as_ref(), &env.domain_class),
            match_value(self.hosting_service.as_ref(), &env.hosting_service),
            match_value(self.asn.as_ref(), &env.asn),
            match_value(self.region.as_ref(), &env.region),
            match_value(self.protocol.as_ref(), &env.protocol),
            match_value(self.tls_version.as_ref(), &env.tls_version),
        ];
        if scores.contains(&None)
            || self
                .method
                .is_some_and(|value| value != query.context.method)
        {
            return None;
        }
        Some(scores.into_iter().flatten().sum::<usize>() + usize::from(self.method.is_some()))
    }
}

#[cfg(test)]
#[path = "prior/test_support.rs"]
mod test_support;

fn match_value<T: PartialEq>(wanted: Option<&T>, actual: &Availability<T>) -> Option<usize> {
    match wanted {
        None => Some(0),
        Some(wanted) => match actual {
            Availability::Available(value) if value == wanted => Some(1),
            _ => None,
        },
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct PriorRegistration {
    pub selector: ColdStartSelector,
    pub prior: ColdStartPrior,
}
