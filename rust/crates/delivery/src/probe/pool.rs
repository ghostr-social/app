//! Probe-pipeline bookkeeping for unresolved HTTP media capabilities.

use crate::manager::retry::RetryBook;
use ghostr_engine::adaptive::ProbeClaimRefusal;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};

mod availability;
mod body;
mod history;
pub(crate) use availability::evidence_needs_head_refresh;
use history::CompletedHeadProbe;

#[derive(Clone, Copy)]
pub(crate) struct ProbeClaimQuery<'a> {
    pub(crate) catalog: &'a Catalog,
    pub(crate) retry: &'a RetryBook,
    pub(crate) post: &'a PostId,
    pub(crate) source: &'a str,
    pub(crate) observed_at_ms: u64,
}

pub(crate) struct MetadataProbePool {
    limit: usize,
    probing: HashMap<PostId, ActiveProbe>,
    probed: HashMap<TransferIdentity, CompletedHeadProbe>,
    deferred: HashSet<TransferIdentity>,
    head_unavailable: HashSet<TransferIdentity>,
}

struct ActiveProbe {
    identity: TransferIdentity,
    result_current: bool,
}

impl ActiveProbe {
    const fn new(identity: TransferIdentity) -> Self {
        Self {
            identity,
            result_current: true,
        }
    }
}

impl MetadataProbePool {
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            probing: HashMap::new(),
            probed: HashMap::new(),
            deferred: HashSet::new(),
            head_unavailable: HashSet::new(),
        }
    }

    pub(crate) fn claim_selected(
        &mut self,
        query: ProbeClaimQuery<'_>,
    ) -> Result<TransferIdentity, ProbeClaimRefusal> {
        let (identity, rearm) = self.probe_identity(&query)?;
        if self.probing.len() >= self.limit {
            return Err(ProbeClaimRefusal::PoolAtCapacity);
        }
        if rearm {
            self.probed.remove(&identity);
        }
        self.probing
            .insert(query.post.clone(), ActiveProbe::new(identity.clone()));
        Ok(identity)
    }

    pub(crate) fn learned_probe(
        &mut self,
        identity: &TransferIdentity,
        stamp: ghostr_engine::representation::HttpGenerationStamp,
        observed_size: bool,
    ) {
        self.probing.remove(identity.post());
        self.deferred.remove(identity);
        self.head_unavailable.remove(identity);
        self.probed.insert(
            identity.clone(),
            CompletedHeadProbe::new(stamp, observed_size),
        );
    }

    pub fn release(&mut self, post: &PostId) {
        self.probing.remove(post);
    }

    pub fn clear(&mut self) {
        self.probing
            .values_mut()
            .for_each(|probe| probe.result_current = false);
        self.probed.clear();
        self.deferred.clear();
        self.head_unavailable.clear();
    }

    pub(crate) fn representation_changed(&mut self, post: &PostId) {
        if let Some(probe) = self.probing.get_mut(post) {
            probe.result_current = false;
        }
        self.probed.retain(|identity, _| identity.post() != post);
        self.deferred.retain(|identity| identity.post() != post);
        self.head_unavailable
            .retain(|identity| identity.post() != post);
    }

    /// Active probes remain counted until completion; only completed
    /// probe-once history follows hot scheduling retention.
    pub(crate) fn retain_history(&mut self, retained: &HashSet<PostId>) {
        self.probed
            .retain(|identity, _| retained.contains(identity.post()));
        self.deferred
            .retain(|identity| retained.contains(identity.post()));
        self.head_unavailable
            .retain(|identity| retained.contains(identity.post()));
    }

    /// Successful HEAD history only; transient probe availability stays a launch-time check.
    pub(crate) fn current_completed_identities(
        &self,
        catalog: &Catalog,
    ) -> HashSet<TransferIdentity> {
        self.probed
            .iter()
            .filter(|(identity, history)| history.current(catalog, identity))
            .map(|(identity, _)| identity.clone())
            .collect()
    }

    pub(crate) fn current_unavailable_identities(
        &self,
        catalog: &Catalog,
    ) -> HashSet<TransferIdentity> {
        self.head_unavailable
            .iter()
            .filter(|identity| {
                catalog
                    .transfer_identity(identity.post(), identity.source().as_str())
                    .as_ref()
                    == Some(*identity)
            })
            .cloned()
            .collect()
    }

    pub(crate) fn active_identities(&self) -> Vec<TransferIdentity> {
        self.probing
            .values()
            .map(|probe| probe.identity.clone())
            .collect()
    }

    pub(crate) fn current_identity(
        &self,
        catalog: &Catalog,
        post: &PostId,
        url: &str,
    ) -> Option<TransferIdentity> {
        let claimed = self.probing.get(post)?;
        if !claimed.result_current {
            return None;
        }
        let current = catalog.transfer_identity(post, url)?;
        (claimed.identity == current).then_some(current)
    }
}

#[cfg(test)]
#[path = "pool_axiom_test.rs"]
pub(crate) mod axiom_test_support;
