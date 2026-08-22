use super::{MetadataProbePool, ProbeClaimQuery};
#[cfg(test)]
use crate::manager::retry::RetryBook;
use crate::manager::retry::Source;
use ghostr_engine::adaptive::ProbeClaimRefusal;
#[cfg(test)]
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
#[cfg(test)]
use ghostr_engine::PostId;

impl MetadataProbePool {
    #[cfg(test)]
    pub(super) fn needed_probe(
        &self,
        catalog: &Catalog,
        retry: &RetryBook,
        post: &PostId,
    ) -> Option<String> {
        if self.probing.contains_key(post)
            || self.probed.contains(post)
            || self.deferred.contains(post)
            || retry.is_cooling(post)
        {
            return None;
        }
        let entry = catalog.lookup(post)?;
        let url = retry.live_urls(post, &entry.meta.urls).into_iter().next()?;
        if entry.planning_total_for(&url).is_some()
            && entry.observed_range_support_for(&url).is_some()
        {
            return None;
        }
        Some(url)
    }

    pub(super) fn probe_identity(
        &self,
        query: &ProbeClaimQuery<'_>,
    ) -> Result<TransferIdentity, ProbeClaimRefusal> {
        self.transient_refusal(query)?;
        let entry = query
            .catalog
            .lookup(query.post)
            .ok_or(ProbeClaimRefusal::CandidateMissing)?;
        source_available(query, &entry.meta.urls)?;
        if entry.planning_total_for(query.source).is_some()
            && entry.observed_range_support_for(query.source).is_some()
        {
            return Err(ProbeClaimRefusal::EvidenceComplete);
        }
        query
            .catalog
            .transfer_identity(query.post, query.source)
            .ok_or(ProbeClaimRefusal::IdentityMissing)
    }

    fn transient_refusal(&self, query: &ProbeClaimQuery<'_>) -> Result<(), ProbeClaimRefusal> {
        let reason = if self.probing.contains_key(query.post) {
            Some(ProbeClaimRefusal::AlreadyProbing)
        } else if self.probed.contains(query.post) {
            Some(ProbeClaimRefusal::AlreadyProbed)
        } else if self.deferred.contains(query.post) {
            Some(ProbeClaimRefusal::DeferredToBody)
        } else if query.retry.is_cooling(query.post) {
            Some(ProbeClaimRefusal::RetryCooling)
        } else {
            None
        };
        reason.map_or(Ok(()), Err)
    }
}

fn source_available(
    query: &ProbeClaimQuery<'_>,
    offered: &[String],
) -> Result<(), ProbeClaimRefusal> {
    if !offered.iter().any(|source| source == query.source) {
        return Err(ProbeClaimRefusal::SourceNotOffered);
    }
    let source = Source::new(query.post.clone(), query.source.to_owned());
    match query.retry.is_retired(&source) {
        true => Err(ProbeClaimRefusal::SourceRetired),
        false => Ok(()),
    }
}
