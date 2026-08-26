use super::*;

impl SourceEvidence {
    fn observed(&self) -> &LearnedFacts {
        &self.observed
    }

    fn authoritative_total(&self) -> Option<u64> {
        self.observed.content_length
    }

    fn observed_range_support(&self) -> Option<bool> {
        self.observed.accept_ranges
    }
}

impl CatalogEntry {
    pub(crate) fn observed_facts_for(&self, source: &str) -> Option<&LearnedFacts> {
        self.evidence.get(source).map(SourceEvidence::observed)
    }

    pub(crate) fn authoritative_total_for(&self, source: &str) -> Option<u64> {
        self.evidence
            .get(source)
            .and_then(SourceEvidence::authoritative_total)
    }

    pub(crate) fn observed_range_support_for(&self, source: &str) -> Option<bool> {
        self.evidence
            .get(source)
            .and_then(SourceEvidence::observed_range_support)
    }
}
