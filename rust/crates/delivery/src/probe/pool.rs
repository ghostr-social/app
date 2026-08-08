//! Probe-pipeline bookkeeping: which unknown-size posts are being
//! HEAD-probed, with bounded concurrency and probe-once memory.

use crate::manager::retry::RetryBook;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::PostId;
use std::collections::HashSet;

pub(crate) struct MetadataProbePool {
    limit: usize,
    probing: HashSet<PostId>,
    probed: HashSet<PostId>,
}

impl MetadataProbePool {
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            probing: HashSet::new(),
            probed: HashSet::new(),
        }
    }

    /// Unknown-size window posts to probe now, bounded by the limit.
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
                self.probing.insert(post.clone());
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

    fn needed_probe(&self, catalog: &Catalog, retry: &RetryBook, post: &PostId) -> Option<String> {
        if self.probing.contains(post) || self.probed.contains(post) || retry.is_cooling(post) {
            return None;
        }
        let entry = catalog.lookup(post)?;
        if entry.total_bytes().is_some() {
            return None;
        }
        retry.live_urls(post, &entry.meta.urls).into_iter().next()
    }
}
