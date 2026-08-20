use super::{Catalog, CatalogEntry};
use crate::evidence::{
    CalibrationContext, CalibrationDimensions, CalibrationLabel, Confidence, Evidence,
    EvidenceSource, EvidenceValue, FieldReliabilityModel,
};

struct LabelOutcome<'a> {
    url: &'a str,
    truths: &'a [EvidenceValue],
    observed_at_ms: u64,
    weight_bps: u16,
}

impl Catalog {
    pub fn field_reliability(&self) -> &FieldReliabilityModel {
        &self.reliability
    }

    pub fn reliability_revision(&self) -> u64 {
        self.reliability_revision
    }

    pub fn replace_field_reliability(&mut self, model: FieldReliabilityModel, now_ms: u64) {
        self.reliability = model;
        self.recalibrate(now_ms);
    }

    pub(super) fn observe_labels(&mut self, labels: Vec<CalibrationLabel>, now_ms: u64) {
        if labels.is_empty() {
            return;
        }
        for label in labels {
            self.reliability.observe(label);
            self.reliability_revision = self.reliability_revision.saturating_add(1);
        }
        self.recalibrate(now_ms);
    }

    pub(super) fn recalibrate(&mut self, now_ms: u64) {
        for entry in self.entries.values_mut() {
            entry.recalibrate(&self.reliability, now_ms);
        }
    }
}

impl CatalogEntry {
    pub(super) fn calibration_labels(
        &self,
        url: &str,
        truths: &[EvidenceValue],
        authority: &EvidenceSource,
        observed_at_ms: u64,
    ) -> Vec<CalibrationLabel> {
        let outcome = LabelOutcome {
            url,
            truths,
            observed_at_ms,
            weight_bps: outcome_weight(authority),
        };
        self.ledger
            .records()
            .iter()
            .filter(|item| label_candidate(item, url, truths, authority))
            .filter_map(|item| label(item, &outcome))
            .collect()
    }

    pub(super) fn recalibrate(&mut self, model: &FieldReliabilityModel, now_ms: u64) {
        for item in self.ledger.records_mut() {
            if !calibratable(&item.source) {
                continue;
            }
            let Some(url) = item.scope.url_value().map(str::to_owned) else {
                continue;
            };
            let context = context(item, &url);
            let estimate = model.estimate(&context, now_ms);
            if estimate.effective_samples_bps > 0 {
                item.confidence = Confidence::new(estimate.mean_bps).unwrap();
            }
        }
    }

    pub(super) fn hash_labels(
        &self,
        digest: &str,
        matches: bool,
        observed_at_ms: u64,
    ) -> Vec<CalibrationLabel> {
        self.ledger
            .records()
            .iter()
            .filter(|item| advertised_digest(item, digest))
            .map(|item| {
                CalibrationLabel::discounted(
                    context(item, item.scope.url_value().unwrap_or_default()),
                    matches,
                    observed_at_ms,
                    10_000,
                )
            })
            .collect()
    }
}

fn label_candidate(
    item: &Evidence<EvidenceValue>,
    url: &str,
    truths: &[EvidenceValue],
    authority: &EvidenceSource,
) -> bool {
    item.is_valid()
        && item.scope.url_value().is_none_or(|value| value == url)
        && item.source.priority() < authority.priority()
        && truths
            .iter()
            .any(|truth| truth.field() == item.value.field())
}

fn label(item: &Evidence<EvidenceValue>, outcome: &LabelOutcome<'_>) -> Option<CalibrationLabel> {
    let truth = outcome
        .truths
        .iter()
        .find(|truth| truth.field() == item.value.field())?;
    Some(CalibrationLabel::discounted(
        context(item, outcome.url),
        item.value == *truth,
        outcome.observed_at_ms,
        outcome.weight_bps,
    ))
}

fn context(item: &Evidence<EvidenceValue>, url: &str) -> CalibrationContext {
    let (issuer, client, origin, kind) = dimensions(&item.source, url);
    let dimensions = CalibrationDimensions::provider(issuer, client, origin, Some(url.to_owned()));
    CalibrationContext::new(dimensions, item.value.field(), kind)
}

fn dimensions(
    source: &EvidenceSource,
    url: &str,
) -> (Option<String>, Option<String>, Option<String>, &'static str) {
    let host = crate::host_stats::host_of(url);
    match source {
        EvidenceSource::Nostr { issuer, client } => {
            (Some(issuer.clone()), client.clone(), host, "nostr")
        }
        EvidenceSource::Head { origin } => (None, None, origin_value(origin), "head"),
        EvidenceSource::Parser { profile } => (None, Some(profile.clone()), host, "parser"),
        EvidenceSource::UrlExtension => (None, None, host, "url_extension"),
        _ => (None, None, host, "outcome"),
    }
}

fn origin_value(value: &str) -> Option<String> {
    crate::host_stats::host_of(value).or_else(|| Some(value.to_owned()))
}

fn calibratable(source: &EvidenceSource) -> bool {
    matches!(
        source,
        EvidenceSource::Nostr { .. }
            | EvidenceSource::Head { .. }
            | EvidenceSource::UrlExtension
            | EvidenceSource::Parser { .. }
    )
}

fn advertised_digest(item: &Evidence<EvidenceValue>, digest: &str) -> bool {
    item.is_valid()
        && matches!(
            &item.value,
            EvidenceValue::AdvertisedHash(value) if value.eq_ignore_ascii_case(digest)
        )
}

fn outcome_weight(source: &EvidenceSource) -> u16 {
    match source {
        EvidenceSource::Head { .. } => 3_000,
        EvidenceSource::Response { .. } => 8_000,
        EvidenceSource::CompleteBytes { .. } | EvidenceSource::Hash { .. } => 10_000,
        EvidenceSource::Parser { .. } => 9_000,
        EvidenceSource::Playback { .. } => 7_000,
        EvidenceSource::Nostr { .. } | EvidenceSource::UrlExtension => 2_000,
    }
}
