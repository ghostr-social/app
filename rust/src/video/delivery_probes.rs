//! Probe-pipeline bookkeeping: which unknown-size posts are being
//! HEAD-probed, with bounded concurrency and probe-once memory.

use crate::engine::catalog::Catalog;
use crate::engine::PostId;
use crate::video::delivery_retry::RetryBook;
use std::collections::HashSet;

pub(crate) struct ProbeBook {
    limit: usize,
    probing: HashSet<PostId>,
    probed: HashSet<PostId>,
}

impl ProbeBook {
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            probing: HashSet::new(),
            probed: HashSet::new(),
        }
    }

    /// Unknown-size window posts to probe now, bounded by the limit.
    /// Returned posts are marked as probing until `finished`. Sources
    /// the retry policy retired are never probed again.
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

    pub fn finished(&mut self, post: &PostId) {
        self.probing.remove(post);
        self.probed.insert(post.clone());
    }

    fn needed_probe(&self, catalog: &Catalog, retry: &RetryBook, post: &PostId) -> Option<String> {
        if self.probing.contains(post) || self.probed.contains(post) {
            return None;
        }
        let entry = catalog.lookup(post)?;
        if entry.total_bytes().is_some() {
            return None;
        }
        retry.live_urls(post, &entry.meta.urls).into_iter().next()
    }
}
