//! Probe-pipeline bookkeeping for unresolved HTTP media capabilities.

use crate::manager::retry::RetryBook;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};

mod availability;

pub(crate) struct MetadataProbePool {
    limit: usize,
    probing: HashMap<PostId, ActiveProbe>,
    probed: HashSet<PostId>,
    deferred: HashSet<PostId>,
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
            probed: HashSet::new(),
            deferred: HashSet::new(),
        }
    }

    /// Window posts with unresolved size or range support, bounded by the limit.
    /// Returned posts are marked as probing until released or learned.
    /// Sources the retry policy retired are never probed again.
    #[cfg(test)]
    pub fn claim(
        &mut self,
        catalog: &Catalog,
        window: &[PostId],
        retry: &RetryBook,
    ) -> Vec<(PostId, String)> {
        let mut claimed = Vec::new();
        for post in window {
            if self.probing.len() >= self.limit {
                break;
            }
            if let Some(url) = self.needed_probe(catalog, retry, post) {
                let identity = catalog
                    .transfer_identity(post, &url)
                    .expect("probe source came from the catalog");
                self.probing
                    .insert(post.clone(), ActiveProbe::new(identity));
                claimed.push((post.clone(), url));
            }
        }
        claimed
    }

    pub(crate) fn claim_selected(
        &mut self,
        catalog: &Catalog,
        retry: &RetryBook,
        post: &PostId,
        source: &str,
    ) -> bool {
        if self.probing.len() >= self.limit || !self.can_probe(catalog, retry, post, source) {
            return false;
        }
        let Some(identity) = catalog.transfer_identity(post, source) else {
            return false;
        };
        self.probing
            .insert(post.clone(), ActiveProbe::new(identity));
        true
    }

    pub fn learned(&mut self, post: &PostId) {
        self.probing.remove(post);
        self.deferred.remove(post);
        self.probed.insert(post.clone());
    }

    pub fn release(&mut self, post: &PostId) {
        self.probing.remove(post);
    }

    pub fn defer_to_body(&mut self, post: &PostId) {
        self.probing.remove(post);
        self.deferred.insert(post.clone());
    }

    pub fn body_finished(&mut self, post: &PostId) {
        self.deferred.remove(post);
    }

    pub(crate) fn reconcile_bodies(&mut self, active: &HashSet<PostId>) {
        self.deferred.retain(|post| active.contains(post));
    }

    pub fn clear(&mut self) {
        self.probing
            .values_mut()
            .for_each(|probe| probe.result_current = false);
        self.probed.clear();
        self.deferred.clear();
    }

    pub(crate) fn representation_changed(&mut self, post: &PostId) {
        if let Some(probe) = self.probing.get_mut(post) {
            probe.result_current = false;
        }
        self.probed.remove(post);
        self.deferred.remove(post);
    }

    /// Active probes remain counted until completion; only completed
    /// probe-once history follows hot scheduling retention.
    pub(crate) fn retain_history(&mut self, retained: &HashSet<PostId>) {
        self.probed.retain(|post| retained.contains(post));
        self.deferred.retain(|post| retained.contains(post));
    }

    /// Successful HEAD history only; transient probe availability stays a launch-time check.
    pub(crate) fn completed_posts(&self) -> &HashSet<PostId> {
        &self.probed
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
