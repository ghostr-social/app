use super::CatalogEntry;
use crate::evidence::{EvidenceTime, EvidenceValidator};
use core::num::NonZeroU64;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LearnedFacts {
    pub content_length: Option<u64>,
    pub accept_ranges: Option<bool>,
    pub host: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpObservation {
    pub(super) facts: LearnedFacts,
    pub(super) content_type: Option<String>,
    pub(super) observed: EvidenceTime,
    pub(super) validator: Option<EvidenceValidator>,
    pub(super) final_url: Option<String>,
    pub(super) generation: Option<crate::representation::HttpGenerationLease>,
    pub(super) request_selection: Option<crate::representation::RequestSelection>,
}

impl HttpObservation {
    pub fn new(
        facts: LearnedFacts,
        content_type: Option<String>,
        observed: impl Into<EvidenceTime>,
        validator: Option<EvidenceValidator>,
    ) -> Self {
        Self {
            facts,
            content_type,
            observed: observed.into(),
            validator,
            final_url: None,
            generation: None,
            request_selection: None,
        }
    }

    pub fn with_final_url(mut self, final_url: impl Into<String>) -> Self {
        self.final_url = Some(final_url.into());
        self
    }

    pub fn with_request_selection(
        mut self,
        selection: Option<crate::representation::RequestSelection>,
    ) -> Self {
        self.request_selection = selection;
        self
    }

    pub(super) fn with_generation(
        mut self,
        generation: crate::representation::HttpGenerationLease,
    ) -> Self {
        self.request_selection = generation.key().request_selection();
        self.generation = Some(generation);
        self
    }

    pub(super) fn legacy(facts: LearnedFacts) -> Self {
        Self::new(facts, None, 0, None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteBytesObservation {
    pub total_bytes: NonZeroU64,
    pub(super) final_url: String,
    pub(super) observed: EvidenceTime,
    pub(super) validator: Option<EvidenceValidator>,
    pub(super) generation: Option<crate::representation::HttpGenerationLease>,
}

impl CompleteBytesObservation {
    pub fn new(
        total_bytes: NonZeroU64,
        final_url: impl Into<String>,
        observed: impl Into<EvidenceTime>,
        validator: Option<EvidenceValidator>,
    ) -> Self {
        Self {
            total_bytes,
            final_url: final_url.into(),
            observed: observed.into(),
            validator,
            generation: None,
        }
    }

    pub fn with_generation(
        mut self,
        generation: crate::representation::HttpGenerationLease,
    ) -> Self {
        self.generation = Some(generation);
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct SourceEvidence {
    advisory: LearnedFacts,
    observed: LearnedFacts,
}

impl LearnedFacts {
    fn merge(&mut self, update: Self) {
        merge_field(&mut self.content_length, update.content_length);
        merge_field(&mut self.accept_ranges, update.accept_ranges);
        merge_field(&mut self.host, update.host);
    }
}

impl SourceEvidence {
    fn learn_head(&mut self, facts: LearnedFacts) {
        self.advisory.merge(facts);
    }

    fn learn_response(&mut self, facts: LearnedFacts) {
        self.observed.merge(facts);
    }

    fn planning_total(&self, declared: Option<u64>) -> Option<u64> {
        self.observed
            .content_length
            .or(self.advisory.content_length)
            .or(declared)
    }
}

fn merge_field<T>(current: &mut Option<T>, update: Option<T>) {
    if update.is_some() {
        *current = update;
    }
}

impl CatalogEntry {
    /// Best-known file size: an observed response beats advisory discovery.
    pub(crate) fn total_bytes(&self) -> Option<u64> {
        self.meta
            .urls
            .first()
            .and_then(|source| self.planning_total_for(source))
            .or(self.meta.size_bytes)
    }

    pub fn planning_total_for(&self, source: &str) -> Option<u64> {
        self.evidence
            .get(source)
            .map_or(self.meta.size_bytes, |evidence| {
                evidence.planning_total(self.meta.size_bytes)
            })
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
}

#[cfg(test)]
#[path = "evidence_axiom_test.rs"]
mod axiom_test_support;
