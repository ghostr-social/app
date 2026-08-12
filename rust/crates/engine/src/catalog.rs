//! Indexed posts: discovery metadata plus facts learned from probing.
//! Pure bookkeeping — probing itself happens elsewhere.

use crate::media_timeline::MediaTimeline;
use crate::playback::{BufferTarget, NetworkConditions, PlaybackObservation};
use crate::representation::{RepresentationBinding, RepresentationGeneration, TransferIdentity};
use crate::video_rendition::VideoRendition;
use crate::{PostId, VideoMeta};
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
    post: PostId,
    pub meta: VideoMeta,
    pub(crate) facts: LearnedFacts,
    binding: RepresentationBinding,
    timeline: Option<MediaTimeline>,
    tail_timeline_needed: bool,
    renditions: renditions::RenditionState,
}

impl CatalogEntry {
    fn new(
        post: PostId,
        meta: VideoMeta,
        variants: Vec<VideoRendition>,
        generation: RepresentationGeneration,
    ) -> Self {
        Self {
            binding: RepresentationBinding::new(post.clone(), &meta, generation),
            post,
            renditions: renditions::RenditionState::new(meta.clone(), variants),
            meta,
            facts: LearnedFacts::default(),
            timeline: None,
            tail_timeline_needed: false,
        }
    }

    fn refresh(
        &mut self,
        meta: VideoMeta,
        variants: Vec<VideoRendition>,
        generation: RepresentationGeneration,
    ) {
        self.renditions = renditions::RenditionState::new(meta.clone(), variants);
        self.switch(meta, generation);
    }

    fn switch(&mut self, meta: VideoMeta, generation: RepresentationGeneration) {
        self.binding = RepresentationBinding::new(self.post.clone(), &meta, generation);
        self.meta = meta;
        self.facts = LearnedFacts::default();
        self.timeline = None;
        self.tail_timeline_needed = false;
    }

    fn selected_meta(
        &self,
        network: NetworkConditions,
        observation: PlaybackObservation,
        target: BufferTarget,
    ) -> Option<VideoMeta> {
        self.renditions
            .select(self.binding.representation(), network, observation, target)
    }

    pub fn binding(&self) -> RepresentationBinding {
        self.binding.clone()
    }

    /// Best-known file size: a probed `Content-Length` beats imeta `size`.
    pub fn total_bytes(&self) -> Option<u64> {
        self.facts.content_length.or(self.meta.size_bytes)
    }

    pub fn timeline(&self) -> Option<&MediaTimeline> {
        self.timeline.as_ref()
    }

    pub fn accepts_byte_ranges(&self) -> Option<bool> {
        self.facts.accept_ranges
    }

    pub fn needs_tail_probe(&self) -> bool {
        self.timeline.is_none() && self.meta.duration_ms.is_none()
    }

    pub(crate) fn needs_timeline_probe(&self) -> bool {
        self.timeline.is_none() && (self.needs_tail_probe() || self.tail_timeline_needed)
    }
}
/// All posts the engine currently knows how to deliver.
#[derive(Debug)]
pub struct Catalog {
    entries: HashMap<PostId, CatalogEntry>,
    next_generation: RepresentationGeneration,
}

impl Default for Catalog {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            next_generation: RepresentationGeneration::first(),
        }
    }
}
impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&PostId) -> bool) {
        self.entries.retain(|post, _| keep(post));
    }

    fn allocate_generation(&mut self) -> RepresentationGeneration {
        let generation = self.next_generation;
        self.next_generation = generation.next();
        generation
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

    pub fn learn_for(&mut self, identity: &TransferIdentity, facts: LearnedFacts) -> bool {
        let Some(entry) = self.entries.get_mut(identity.post()) else {
            return false;
        };
        if entry.binding.transfer(identity.source().as_str()).as_ref() != Some(identity) {
            return false;
        }
        entry.facts.merge(facts);
        true
    }

    pub fn binding(&self, post: &PostId) -> Option<RepresentationBinding> {
        self.lookup(post).map(CatalogEntry::binding)
    }

    pub fn transfer_identity(&self, post: &PostId, url: &str) -> Option<TransferIdentity> {
        self.lookup(post)?.binding.transfer(url)
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
}

mod bitrate;
mod renditions;
mod timeline;
