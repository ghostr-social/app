use super::{MetadataProbePool, ProbeClaimQuery};
#[cfg(test)]
use crate::manager::retry::RetryBook;
use crate::manager::retry::Source;
use ghostr_engine::adaptive::ProbeClaimRefusal;
#[cfg(test)]
use ghostr_engine::catalog::Catalog;
use ghostr_engine::evidence::{EvidenceAssessment, EvidenceField};
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
        if self.probing.contains_key(post) || self.deferred.contains(post) || retry.is_cooling(post)
        {
            return None;
        }
        let entry = catalog.lookup(post)?;
        let url = retry.live_urls(post, &entry.meta.urls).into_iter().next()?;
        let identity = catalog.transfer_identity(post, &url)?;
        if self
            .probed
            .get(&identity)
            .is_some_and(|history| history.current(catalog, &identity))
        {
            return None;
        }
        if evidence_complete(&entry.evidence_assessment_for(&url, 0)) {
            return None;
        }
        Some(url)
    }

    pub(super) fn probe_identity(
        &self,
        query: &ProbeClaimQuery<'_>,
    ) -> Result<(TransferIdentity, bool), ProbeClaimRefusal> {
        let entry = query
            .catalog
            .lookup(query.post)
            .ok_or(ProbeClaimRefusal::CandidateMissing)?;
        source_available(query, &entry.meta.urls)?;
        let evidence = entry.evidence_assessment_for(query.source, query.observed_at_ms);
        let identity = query
            .catalog
            .transfer_identity(query.post, query.source)
            .ok_or(ProbeClaimRefusal::IdentityMissing)?;
        let had_size = entry.planning_total_for(query.source).is_some();
        let history = self.probed.get(&identity);
        let generation_changed =
            history.is_some_and(|completed| !completed.current(query.catalog, &identity));
        let size_refresh = evidence_needs_head_refresh(&evidence, had_size)
            && history.map_or(true, |completed| {
                generation_changed || completed.observed_size()
            });
        let rearm = generation_changed || size_refresh;
        self.transient_refusal(query, &identity, rearm)?;
        if evidence_complete(&evidence) {
            return Err(ProbeClaimRefusal::EvidenceComplete);
        }
        Ok((identity, rearm))
    }

    fn transient_refusal(
        &self,
        query: &ProbeClaimQuery<'_>,
        identity: &TransferIdentity,
        rearm: bool,
    ) -> Result<(), ProbeClaimRefusal> {
        let reason = if self.probing.contains_key(query.post) {
            Some(ProbeClaimRefusal::AlreadyProbing)
        } else if self.probed.contains_key(identity) && !rearm {
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

fn evidence_complete(assessment: &EvidenceAssessment) -> bool {
    assessment.size.exact.is_some() && assessment.value(EvidenceField::RangeSupport).is_some()
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
pub(crate) fn evidence_needs_head_refresh(assessment: &EvidenceAssessment, had_size: bool) -> bool {
    assessment.stale.contains(&EvidenceField::Size)
        || had_size && assessment.missing.contains(&EvidenceField::Size)
}
