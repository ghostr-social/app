use super::super::HttpAuthority;
use crate::catalog::{CatalogEntry, HttpObservation};
use crate::evidence::EvidenceTime;
use crate::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease, HttpGenerationStamp,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::catalog) struct HttpGenerationRecord {
    stamp: HttpGenerationStamp,
    observed: EvidenceTime,
}

impl CatalogEntry {
    pub(super) fn accept_http_generation(
        &mut self,
        url: &str,
        observation: &HttpObservation,
        authority: HttpAuthority,
        observed: EvidenceTime,
    ) -> Option<()> {
        match authority {
            HttpAuthority::CompleteBytes => self.accept_complete_generation(url, observation),
            HttpAuthority::Head if observation.validator.is_none() => {
                self.accept_validatorless_head(url, observation, observed)
            }
            HttpAuthority::Head | HttpAuthority::Response => {
                self.accept_trusted_generation(url, observation, observed)
            }
        }
    }

    pub(in crate::catalog) fn http_generation_stamp(
        &self,
        url: &str,
    ) -> Option<HttpGenerationStamp> {
        Some(self.http_generations.get(url)?.stamp.clone())
    }

    pub(in crate::catalog) fn reject_response_generation(
        &mut self,
        url: &str,
        final_url: &str,
        validator: Option<crate::evidence::EvidenceValidator>,
        observed: EvidenceTime,
    ) -> Option<HttpGenerationAuthority> {
        if validator.is_none() {
            return self
                .http_generations
                .get(url)
                .map(|current| current.stamp.authority().clone());
        }
        let candidate = HttpGenerationKey::try_new(final_url, validator).ok()?;
        if let Some(current) = self.http_generations.get(url).cloned() {
            if current.stamp.key() == &candidate {
                return Some(current.stamp.authority().clone());
            }
            if !observed.is_after(current.observed) {
                return None;
            }
        }
        self.install_unknown_generation(url, candidate, observed)
            .map(|stamp| stamp.authority().clone())
    }

    fn accept_validatorless_head(
        &mut self,
        url: &str,
        observation: &HttpObservation,
        observed: EvidenceTime,
    ) -> Option<()> {
        let final_url = observation.final_url.as_deref().unwrap_or(url);
        let candidate = HttpGenerationKey::try_new(final_url, None).ok()?;
        if let Some(current) = self.http_generations.get(url).cloned() {
            if !observed.is_after(current.observed) {
                return None;
            }
            if current.stamp.key().final_url() == final_url {
                return Some(());
            }
        }
        self.install_unknown_generation(url, candidate, observed)?;
        Some(())
    }

    fn accept_trusted_generation(
        &mut self,
        url: &str,
        observation: &HttpObservation,
        observed: EvidenceTime,
    ) -> Option<()> {
        let final_url = observation.final_url.as_deref().unwrap_or(url);
        let key = HttpGenerationKey::try_new(final_url, observation.validator.clone()).ok()?;
        if let Some(current) = self.http_generations.get(url).cloned() {
            if matches_key(current.stamp.authority(), &key) {
                if observed.is_after(current.observed) {
                    self.http_generations.get_mut(url)?.observed = observed;
                }
                return Some(());
            }
            if !observed.is_after(current.observed) {
                return None;
            }
            return self.install_trusted_generation(url, key, observed, true);
        }
        self.install_trusted_generation(url, key, observed, false)
    }

    fn install_trusted_generation(
        &mut self,
        url: &str,
        key: HttpGenerationKey,
        observed: EvidenceTime,
        invalidate: bool,
    ) -> Option<()> {
        let epoch = self.next_http_generation;
        self.next_http_generation = epoch.checked_add(1)?;
        let lease = HttpGenerationLease::try_new(key.clone(), epoch).ok()?;
        let authority = HttpGenerationAuthority::Trusted(lease);
        if invalidate {
            let invalidation =
                self.ledger
                    .replace_url_generation_at(url, key.validator().cloned(), observed);
            self.apply_invalidation(invalidation);
            self.evidence.remove(url);
        } else {
            self.ledger
                .adopt_url_generation_at(url, key.validator().cloned(), observed);
        }
        self.http_generations.insert(
            url.to_owned(),
            HttpGenerationRecord {
                stamp: HttpGenerationStamp::new(key, authority),
                observed,
            },
        );
        Some(())
    }

    fn install_unknown_generation(
        &mut self,
        url: &str,
        key: HttpGenerationKey,
        observed: EvidenceTime,
    ) -> Option<HttpGenerationStamp> {
        let epoch =
            crate::representation::HttpGenerationEpoch::try_new(self.next_http_generation).ok()?;
        self.next_http_generation = self.next_http_generation.checked_add(1)?;
        let authority = HttpGenerationAuthority::Unknown(epoch);
        let stamp = HttpGenerationStamp::new(key, authority);
        let invalidation = self.ledger.revoke_url_generation_at(url, observed);
        self.apply_invalidation(invalidation);
        self.evidence.remove(url);
        self.http_generations.insert(
            url.to_owned(),
            HttpGenerationRecord {
                stamp: stamp.clone(),
                observed,
            },
        );
        Some(stamp)
    }

    fn accept_complete_generation(&self, url: &str, observation: &HttpObservation) -> Option<()> {
        let Some(current) = self.http_generations.get(url) else {
            return (observation.generation.is_none()
                && self.ledger.current_validator(url) == observation.validator.as_ref())
            .then_some(());
        };
        let lease = observation.generation.as_ref()?;
        let trusted = matches!(
            current.stamp.authority(),
            HttpGenerationAuthority::Trusted(value) if *value == *lease
        );
        let final_url = observation.final_url.as_deref().unwrap_or(url);
        (trusted
            && lease.key().final_url() == final_url
            && lease.key().validator() == observation.validator.as_ref())
        .then_some(())
    }
}

fn matches_key(authority: &HttpGenerationAuthority, key: &HttpGenerationKey) -> bool {
    matches!(authority, HttpGenerationAuthority::Trusted(lease) if lease.key() == key)
}
