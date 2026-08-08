//! Indexed posts: discovery metadata plus facts learned from probing.
//! Pure bookkeeping — probing itself happens elsewhere.

use crate::{EngineParams, PostId, VideoMeta};
use std::collections::HashMap;

/// Facts the engine learns about a video after discovery (HEAD probes,
/// observed transfers). Absent fields are simply not yet learned.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LearnedFacts {
    pub content_length: Option<u64>,
    pub accept_ranges: Option<bool>,
    pub host: Option<String>,
}

impl LearnedFacts {
    fn merge(&mut self, update: LearnedFacts) {
        if update.content_length.is_some() {
            self.content_length = update.content_length;
        }
        if update.accept_ranges.is_some() {
            self.accept_ranges = update.accept_ranges;
        }
        if update.host.is_some() {
            self.host = update.host;
        }
    }
}

/// One catalogued post: what discovery said plus what probing taught us.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogEntry {
    pub meta: VideoMeta,
    pub(crate) facts: LearnedFacts,
}

impl CatalogEntry {
    fn new(meta: VideoMeta) -> Self {
        Self {
            meta,
            facts: LearnedFacts::default(),
        }
    }

    /// Best-known file size: a probed `Content-Length` beats imeta `size`.
    pub fn total_bytes(&self) -> Option<u64> {
        self.facts.content_length.or(self.meta.size_bytes)
    }
}

/// All posts the engine currently knows how to deliver.
#[derive(Debug, Default)]
pub struct Catalog {
    entries: HashMap<PostId, CatalogEntry>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a post or refreshes its discovery metadata. Learned facts
    /// survive metadata refreshes.
    pub fn upsert(&mut self, post: PostId, meta: VideoMeta) {
        match self.entries.get_mut(&post) {
            Some(entry) => entry.meta = meta,
            None => {
                self.entries.insert(post, CatalogEntry::new(meta));
            }
        }
    }

    /// Merges freshly learned facts into a known post. Returns `false`
    /// when the post is not catalogued.
    pub fn learn(&mut self, post: &PostId, facts: LearnedFacts) -> bool {
        match self.entries.get_mut(post) {
            Some(entry) => {
                entry.facts.merge(facts);
                true
            }
            None => false,
        }
    }

    pub fn lookup(&self, post: &PostId) -> Option<&CatalogEntry> {
        self.entries.get(post)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Bits per second from best-known size and duration; falls back to
    /// the assumed bitrate when either is unknown or degenerate.
    pub(crate) fn estimated_bitrate(&self, post: &PostId, params: &EngineParams) -> u64 {
        self.lookup(post)
            .and_then(measured_bitrate)
            .unwrap_or(params.assumed_bitrate_bps)
    }
}

fn measured_bitrate(entry: &CatalogEntry) -> Option<u64> {
    let bytes = entry.total_bytes()?;
    let duration_ms = entry.meta.duration_ms.filter(|ms| *ms > 0)?;
    Some(bytes.saturating_mul(8).saturating_mul(1_000) / duration_ms)
}
