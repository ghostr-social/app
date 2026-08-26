use super::*;

impl EvidenceLedger {
    pub(crate) fn observe_validator(
        &mut self,
        url: &str,
        validator: EvidenceValidator,
        observed_at_ms: u64,
    ) -> EvidenceInvalidation {
        self.observe_validator_at(url, validator, observed_at_ms.into())
            .unwrap_or_default()
    }

    fn observe_validator_at(
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
        let invalidation = self.invalidate_url_generation(url, &validator, observed.observed_at_ms);
        self.validators.insert(url.to_owned(), validator);
        Some(invalidation)
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
                super::super::note_invalidation(&mut result, item, observed_at_ms);
            }
        }
        result
    }
}
