use super::{HttpAuthority, HttpLearning};
use crate::catalog::{CatalogEntry, HttpObservation};
use crate::evidence::{Confidence, Evidence, EvidenceScope, EvidenceSource, EvidenceValue};

mod generation;
mod validator;
pub(in crate::catalog) use generation::HttpGenerationRecord;

impl CatalogEntry {
    pub(in crate::catalog) fn learn_action_http(
        &mut self,
        url: &str,
        observation: HttpObservation,
    ) -> Option<HttpLearning> {
        let observation =
            self.normalize_http_observation(url, HttpAuthority::Response, observation)?;
        let observed = observation.observed;
        let source = http_source(url, &observation, HttpAuthority::Response);
        let labels = self.calibration_labels(
            url,
            &http_truths(&observation),
            &source,
            observed.observed_at_ms,
        );
        self.retain_response(url, observation.facts.clone());
        self.record_http_fields(url, observation, HttpAuthority::Response);
        Some(HttpLearning {
            observed_at_ms: observed.observed_at_ms,
            labels,
        })
    }

    pub(in crate::catalog) fn learn_http(
        &mut self,
        url: &str,
        observation: HttpObservation,
        authority: HttpAuthority,
    ) -> Option<HttpLearning> {
        if authority == HttpAuthority::Response && observation.validator.is_none() {
            return None;
        }
        let observation = self.normalize_http_observation(url, authority, observation)?;
        let observed = observation.observed;
        self.accept_http_generation(url, &observation, authority, observed)?;
        let observed_at_ms = observed.observed_at_ms;
        let source = http_source(url, &observation, authority);
        let labels =
            self.calibration_labels(url, &http_truths(&observation), &source, observed_at_ms);
        match authority {
            HttpAuthority::Head => self.retain_head(url, observation.facts.clone()),
            HttpAuthority::Response | HttpAuthority::CompleteBytes => {
                self.retain_response(url, observation.facts.clone())
            }
        }
        self.record_http_fields(url, observation, authority);
        Some(HttpLearning {
            observed_at_ms,
            labels,
        })
    }

    pub(in crate::catalog) fn record_integrity(
        &mut self,
        digest: &str,
        origin: &str,
        observed: crate::evidence::EvidenceTime,
    ) {
        self.ledger.record(Evidence::new_at(
            EvidenceValue::IntegrityMatch {
                digest: digest.to_ascii_lowercase(),
                matches: true,
            },
            EvidenceSource::hash(origin),
            observed,
            Confidence::certain(),
            EvidenceScope::ImmutableBytes(digest.to_ascii_lowercase()),
        ));
    }

    fn record_http_fields(
        &mut self,
        url: &str,
        observation: HttpObservation,
        authority: HttpAuthority,
    ) {
        let source = http_source(url, &observation, authority);
        let scope = observation.validator.clone().map_or_else(
            || EvidenceScope::url(url),
            |validator| EvidenceScope::validated(url, validator),
        );
        let confidence = http_confidence(authority);
        for value in http_truths(&observation) {
            self.ledger.record(Evidence::new_at(
                value,
                source.clone(),
                observation.observed,
                confidence,
                scope.clone(),
            ));
        }
    }

    fn accept_http_time(
        &mut self,
        url: &str,
        authority: HttpAuthority,
        requested: crate::evidence::EvidenceTime,
    ) -> Option<crate::evidence::EvidenceTime> {
        let observed = match requested.observed_at_ms {
            0 => crate::evidence::EvidenceTime::from(self.evidence_clock_ms.saturating_add(1)),
            _ => requested,
        };
        if self
            .http_clocks
            .get(&(url.to_owned(), authority))
            .is_some_and(|current| !observed.is_after(*current))
        {
            return None;
        }
        self.http_clocks
            .insert((url.to_owned(), authority), observed);
        self.evidence_clock_ms = self.evidence_clock_ms.max(observed.observed_at_ms);
        Some(observed)
    }

    fn normalize_http_observation(
        &mut self,
        url: &str,
        authority: HttpAuthority,
        mut observation: HttpObservation,
    ) -> Option<HttpObservation> {
        observation.observed = self.accept_http_time(url, authority, observation.observed)?;
        Some(observation)
    }
}

fn http_source(
    url: &str,
    observation: &HttpObservation,
    authority: HttpAuthority,
) -> EvidenceSource {
    let origin = observation
        .facts
        .host
        .clone()
        .unwrap_or_else(|| url.to_owned());
    match authority {
        HttpAuthority::Head => EvidenceSource::head(origin),
        HttpAuthority::Response => EvidenceSource::response(origin),
        HttpAuthority::CompleteBytes => EvidenceSource::CompleteBytes { origin },
    }
}

fn http_truths(observation: &HttpObservation) -> Vec<EvidenceValue> {
    let mut values = Vec::new();
    if let Some(bytes) = observation.facts.content_length.filter(|value| *value > 0) {
        values.push(EvidenceValue::SizeBytes(bytes));
    }
    if let Some(ranges) = observation.facts.accept_ranges {
        values.push(EvidenceValue::RangeSupport(ranges));
    }
    if let Some(mime) = observation.content_type.as_deref() {
        values.push(EvidenceValue::Mime(normalize_mime(mime)));
    }
    values
}

fn http_confidence(authority: HttpAuthority) -> Confidence {
    let basis_points = match authority {
        HttpAuthority::Head => 6_500,
        HttpAuthority::Response => 9_000,
        HttpAuthority::CompleteBytes => 10_000,
    };
    Confidence::new(basis_points).unwrap()
}

fn normalize_mime(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or(value)
        .trim()
        .to_ascii_lowercase()
}
