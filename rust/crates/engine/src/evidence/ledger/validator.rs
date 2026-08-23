use super::{EvidenceInvalidation, EvidenceLedger};
use crate::evidence::{
    EvidenceScope, EvidenceSource, EvidenceTime, EvidenceValidator, EvidenceValue,
};

impl EvidenceLedger {
    pub fn observe_validator(
        &mut self,
        url: &str,
        validator: EvidenceValidator,
        observed_at_ms: u64,
    ) -> EvidenceInvalidation {
        self.observe_validator_at(url, validator, observed_at_ms.into())
            .unwrap_or_default()
    }

    pub(crate) fn observe_validator_at(
        &mut self,
        url: &str,
        validator: EvidenceValidator,
        observed: EvidenceTime,
    ) -> Option<EvidenceInvalidation> {
        if self.validators.get(url) == Some(&validator) {
            self.advance_validator_time(url, observed);
            return Some(EvidenceInvalidation::default());
        }
        if self
            .validator_times
            .get(url)
            .is_some_and(|current| !observed.is_after(*current))
        {
            return None;
        }
        self.validator_times.insert(url.to_owned(), observed);
        self.validators.insert(url.to_owned(), validator.clone());
        Some(self.invalidate_url_generation(url, &validator, observed.observed_at_ms))
    }

    pub(crate) fn scope_for_url(&self, url: &str) -> EvidenceScope {
        self.validators.get(url).cloned().map_or_else(
            || EvidenceScope::url(url),
            |validator| EvidenceScope::validated(url, validator),
        )
    }

    pub(crate) fn current_validator(&self, url: &str) -> Option<&EvidenceValidator> {
        self.validators.get(url)
    }

    pub(crate) fn replace_url_generation_at(
        &mut self,
        url: &str,
        validator: Option<EvidenceValidator>,
        observed: EvidenceTime,
    ) -> EvidenceInvalidation {
        self.validator_times.insert(url.to_owned(), observed);
        match validator {
            Some(validator) => {
                self.validators.insert(url.to_owned(), validator);
            }
            None => {
                self.validators.remove(url);
            }
        }
        self.invalidate_url(url, observed.observed_at_ms)
    }

    pub(crate) fn adopt_url_generation_at(
        &mut self,
        url: &str,
        validator: Option<EvidenceValidator>,
        observed: EvidenceTime,
    ) {
        self.validator_times.insert(url.to_owned(), observed);
        match validator {
            Some(validator) => {
                self.validators.insert(url.to_owned(), validator);
            }
            None => {
                self.validators.remove(url);
            }
        }
    }

    pub(crate) fn revoke_url_generation_at(
        &mut self,
        url: &str,
        observed: EvidenceTime,
    ) -> EvidenceInvalidation {
        self.validator_times.insert(url.to_owned(), observed);
        self.validators.remove(url);
        self.invalidate_url(url, observed.observed_at_ms)
    }

    fn advance_validator_time(&mut self, url: &str, observed: EvidenceTime) {
        let advances = self
            .validator_times
            .get(url)
            .is_none_or(|current| observed.is_after(*current));
        if advances {
            self.validator_times.insert(url.to_owned(), observed);
        }
    }

    fn invalidate_url_generation(
        &mut self,
        url: &str,
        current: &EvidenceValidator,
        observed_at_ms: u64,
    ) -> EvidenceInvalidation {
        let mut result = EvidenceInvalidation::default();
        for item in &mut self.records {
            if url_derived(item, url) && item.validator.as_ref() != Some(current) {
                super::note_invalidation(&mut result, item, observed_at_ms);
            }
        }
        result
    }

    fn invalidate_url(&mut self, url: &str, observed_at_ms: u64) -> EvidenceInvalidation {
        let mut result = EvidenceInvalidation::default();
        for item in &mut self.records {
            if url_derived(item, url) {
                super::note_invalidation(&mut result, item, observed_at_ms);
            }
        }
        result
    }
}

fn url_derived(item: &crate::evidence::Evidence<EvidenceValue>, url: &str) -> bool {
    item.scope.url_value() == Some(url)
        && !matches!(
            item.source,
            EvidenceSource::Nostr { .. } | EvidenceSource::UrlExtension
        )
}
