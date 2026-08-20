//! Indexed posts: discovery metadata plus facts learned from probing.
//! Pure bookkeeping — probing itself happens elsewhere.

use crate::media_timeline::MediaTimeline;
use crate::playback::{BufferTarget, NetworkConditions, PlaybackObservation};
use crate::representation::{RepresentationBinding, RepresentationGeneration, TransferIdentity};
use crate::video_rendition::VideoRendition;
use crate::{PostId, VideoMeta};
use std::collections::{BTreeSet, HashMap};

mod evidence;
use evidence::SourceEvidence;
pub use evidence::{HttpObservation, LearnedFacts};
mod calibration;
mod compatibility;
mod ledger;
mod observation;
mod persistence;
mod playback_evidence;
pub use persistence::CatalogEvidenceState;
pub use playback_evidence::PlaybackEvidence;

/// One catalogued post: what discovery said plus what probing taught us.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogEntry {
    post: PostId,
    pub meta: VideoMeta,
    evidence: HashMap<String, SourceEvidence>,
    ledger: crate::evidence::EvidenceLedger,
    evidence_clock_ms: u64,
    quarantined: bool,
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
        let mut entry = Self {
            binding: RepresentationBinding::new(post.clone(), &meta, generation),
            post,
            renditions: renditions::RenditionState::new(meta.clone(), variants),
            meta,
            evidence: HashMap::new(),
            ledger: crate::evidence::EvidenceLedger::default(),
            evidence_clock_ms: 0,
            quarantined: false,
            timeline: None,
            tail_timeline_needed: false,
        };
        entry.seed_declared_evidence();
        entry
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
        self.evidence.clear();
        self.ledger = crate::evidence::EvidenceLedger::default();
        self.evidence_clock_ms = 0;
        self.quarantined = false;
        self.timeline = None;
        self.tail_timeline_needed = false;
        self.seed_declared_evidence();
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

    pub fn evidence(&self) -> &crate::evidence::EvidenceLedger {
        &self.ledger
    }

    pub fn is_quarantined(&self) -> bool {
        self.quarantined
    }

    pub fn timeline(&self) -> Option<&MediaTimeline> {
        self.timeline.as_ref()
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
    reliability: crate::evidence::FieldReliabilityModel,
    reliability_revision: u64,
    digest_claims: HashMap<String, BTreeSet<PostId>>,
    quarantined_digests: BTreeSet<String>,
    next_generation: RepresentationGeneration,
}

impl Default for Catalog {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            reliability: crate::evidence::FieldReliabilityModel::default(),
            reliability_revision: 0,
            digest_claims: HashMap::new(),
            quarantined_digests: BTreeSet::new(),
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
