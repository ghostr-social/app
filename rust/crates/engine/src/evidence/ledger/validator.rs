use super::{EvidenceInvalidation, EvidenceLedger};
use crate::evidence::{
    EvidenceScope, EvidenceSource, EvidenceTime, EvidenceValidator, EvidenceValue,
};

impl EvidenceLedger {
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

#[cfg(test)]
#[path = "validator/test_support.rs"]
mod test_support;

fn url_derived(item: &crate::evidence::Evidence<EvidenceValue>, url: &str) -> bool {
    item.scope.url_value() == Some(url)
        && !matches!(
            item.source,
            EvidenceSource::Nostr { .. } | EvidenceSource::UrlExtension
        )
}
