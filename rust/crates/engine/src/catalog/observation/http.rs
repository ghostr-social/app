use super::{HttpAuthority, HttpLearning};
use crate::catalog::{CatalogEntry, HttpObservation};
use crate::evidence::{Confidence, Evidence, EvidenceScope, EvidenceSource, EvidenceValue};

impl CatalogEntry {
    pub(in crate::catalog) fn learn_http(
        &mut self,
        url: &str,
        mut observation: HttpObservation,
        authority: HttpAuthority,
    ) -> HttpLearning {
        let observed_at_ms = self.next_observation_time(observation.observed_at_ms);
        observation.observed_at_ms = observed_at_ms;
        self.apply_validator(url, observation.validator.as_ref(), observed_at_ms);
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
        HttpLearning {
            observed_at_ms,
            labels,
        }
    }

    pub(in crate::catalog) fn record_integrity(
        &mut self,
        digest: &str,
        origin: &str,
        observed_at_ms: u64,
    ) {
        self.ledger.record(Evidence::new(
            EvidenceValue::IntegrityMatch {
                digest: digest.to_ascii_lowercase(),
                matches: true,
            },
            EvidenceSource::hash(origin),
            observed_at_ms,
            Confidence::certain(),
            EvidenceScope::ImmutableBytes(digest.to_ascii_lowercase()),
        ));
    }

    fn apply_validator(
        &mut self,
        url: &str,
        validator: Option<&crate::evidence::EvidenceValidator>,
        observed_at_ms: u64,
    ) {
        let Some(validator) = validator else { return };
        let invalidation = self
            .ledger
            .observe_validator(url, validator.clone(), observed_at_ms);
        if invalidation.structural_evidence {
            self.timeline = None;
            self.tail_timeline_needed = false;
        }
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
            self.ledger.record(Evidence::new(
                value,
                source.clone(),
                observation.observed_at_ms,
                confidence,
                scope.clone(),
            ));
        }
    }

    fn next_observation_time(&mut self, requested: u64) -> u64 {
        let observed = requested.max(self.evidence_clock_ms.saturating_add(1));
        self.evidence_clock_ms = observed;
        observed
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
