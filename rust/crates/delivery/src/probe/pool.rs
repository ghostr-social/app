//! Probe-pipeline bookkeeping for unresolved HTTP media capabilities.

use crate::manager::retry::RetryBook;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};

pub(crate) struct MetadataProbePool {
    limit: usize,
    probing: HashMap<PostId, TransferIdentity>,
    probed: HashSet<PostId>,
}

impl MetadataProbePool {
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            probing: HashMap::new(),
            probed: HashSet::new(),
        }
    }

    /// Window posts with unresolved size or range support, bounded by the limit.
    /// Returned posts are marked as probing until released or learned.
    /// Sources the retry policy retired are never probed again.
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
                self.probing.insert(post.clone(), identity);
                claimed.push((post.clone(), url));
            }
        }
        claimed
    }

    pub fn learned(&mut self, post: &PostId) {
        self.probing.remove(post);
        self.probed.insert(post.clone());
    }

    pub fn release(&mut self, post: &PostId) {
        self.probing.remove(post);
    }

    pub fn clear(&mut self) {
        self.probing.clear();
        self.probed.clear();
    }

    pub(crate) fn representation_changed(&mut self, post: &PostId) {
        self.probing.remove(post);
        self.probed.remove(post);
    }

    /// Active probes remain counted until completion; only completed
    /// probe-once history follows hot scheduling retention.
    pub(crate) fn retain_history(&mut self, retained: &HashSet<PostId>) {
        self.probed.retain(|post| retained.contains(post));
    }

    pub(crate) fn current_identity(
        &self,
        catalog: &Catalog,
        post: &PostId,
        url: &str,
    ) -> Option<TransferIdentity> {
        let claimed = self.probing.get(post)?;
        let current = catalog.transfer_identity(post, url)?;
        (claimed == &current).then_some(current)
    }

    fn needed_probe(&self, catalog: &Catalog, retry: &RetryBook, post: &PostId) -> Option<String> {
        if self.probing.contains_key(post) || self.probed.contains(post) || retry.is_cooling(post) {
            return None;
        }
        let entry = catalog.lookup(post)?;
        if entry.total_bytes().is_some() && entry.accepts_byte_ranges().is_some() {
            return None;
        }
        retry.live_urls(post, &entry.meta.urls).into_iter().next()
    }
}
