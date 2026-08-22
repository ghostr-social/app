use super::CatalogEntry;
use crate::evidence::EvidenceValidator;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LearnedFacts {
    pub content_length: Option<u64>,
    pub accept_ranges: Option<bool>,
    pub host: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpObservation {
    pub facts: LearnedFacts,
    pub content_type: Option<String>,
    pub observed_at_ms: u64,
    pub validator: Option<EvidenceValidator>,
}

impl HttpObservation {
    pub fn new(
        facts: LearnedFacts,
        content_type: Option<String>,
        observed_at_ms: u64,
        validator: Option<EvidenceValidator>,
    ) -> Self {
        Self {
            facts,
            content_type,
            observed_at_ms,
            validator,
        }
    }

    pub(super) fn legacy(facts: LearnedFacts) -> Self {
        Self::new(facts, None, 0, None)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct SourceEvidence {
    advisory: LearnedFacts,
    observed: LearnedFacts,
}

impl LearnedFacts {
    fn merge(&mut self, update: LearnedFacts) {
        merge_field(&mut self.content_length, update.content_length);
        merge_field(&mut self.accept_ranges, update.accept_ranges);
        merge_field(&mut self.host, update.host);
    }
}

impl SourceEvidence {
    pub(super) fn learn_head(&mut self, facts: LearnedFacts) {
        self.advisory.merge(facts);
    }

    pub(super) fn learn_response(&mut self, facts: LearnedFacts) {
        self.observed.merge(facts);
    }

    pub(super) fn planning_total(&self, declared: Option<u64>) -> Option<u64> {
        self.observed
            .content_length
            .or(self.advisory.content_length)
            .or(declared)
    }

    pub(super) fn authoritative_total(&self) -> Option<u64> {
        self.observed.content_length
    }

    pub(super) fn observed_range_support(&self) -> Option<bool> {
        self.observed.accept_ranges
    }

    #[cfg(test)]
    pub(super) fn observed(&self) -> &LearnedFacts {
        &self.observed
    }
}

fn merge_field<T>(current: &mut Option<T>, update: Option<T>) {
    if update.is_some() {
        *current = update;
    }
}

impl CatalogEntry {
    /// Best-known file size: an observed response beats advisory discovery.
    pub fn total_bytes(&self) -> Option<u64> {
        self.meta
            .urls
            .first()
            .and_then(|source| self.planning_total_for(source))
            .or(self.meta.size_bytes)
    }

    pub fn accepts_byte_ranges(&self) -> Option<bool> {
        self.meta
            .urls
            .first()
            .and_then(|source| self.observed_range_support_for(source))
    }

    pub fn planning_total_for(&self, source: &str) -> Option<u64> {
        self.evidence
            .get(source)
            .map_or(self.meta.size_bytes, |evidence| {
                evidence.planning_total(self.meta.size_bytes)
            })
    }

    pub fn authoritative_total_for(&self, source: &str) -> Option<u64> {
        self.evidence
            .get(source)
            .and_then(SourceEvidence::authoritative_total)
    }

    pub fn observed_range_support_for(&self, source: &str) -> Option<bool> {
        self.evidence
            .get(source)
            .and_then(SourceEvidence::observed_range_support)
    }

    pub(super) fn retain_head(&mut self, source: &str, facts: LearnedFacts) {
        self.evidence
            .entry(source.to_owned())
            .or_default()
            .learn_head(facts);
    }

    pub(super) fn retain_response(&mut self, source: &str, facts: LearnedFacts) {
        self.evidence
            .entry(source.to_owned())
            .or_default()
            .learn_response(facts);
    }

    #[cfg(test)]
    pub(crate) fn observed_facts_for(&self, source: &str) -> Option<&LearnedFacts> {
        self.evidence.get(source).map(SourceEvidence::observed)
    }
}
