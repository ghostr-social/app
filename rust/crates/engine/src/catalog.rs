//! Indexed posts: discovery metadata plus facts learned from probing.
//! Pure bookkeeping — probing itself happens elsewhere.

use crate::media_timeline::MediaTimeline;
use crate::representation::{RepresentationBinding, RepresentationGeneration, TransferIdentity};
use crate::video_rendition::VideoRendition;
use crate::{PostId, PreviewDescriptor, VideoMeta};
use std::collections::{BTreeSet, HashMap, HashSet};

mod evidence;
use evidence::SourceEvidence;
pub use evidence::{CompleteBytesObservation, HttpObservation, LearnedFacts};
mod calibration;
mod compatibility;
mod ledger;
mod observation;
mod persistence;
mod playback_evidence;
pub use persistence::CatalogEvidenceState;
pub use playback_evidence::PlaybackEvidence;
pub use renditions::RenditionSelection;

/// One catalogued post: what discovery said plus what probing taught us.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogEntry {
    post: PostId,
    pub meta: VideoMeta,
    evidence: HashMap<String, SourceEvidence>,
    ledger: crate::evidence::EvidenceLedger,
    evidence_clock_ms: u64,
    http_clocks: HashMap<(String, observation::HttpAuthority), crate::evidence::EvidenceTime>,
    http_generations: HashMap<String, observation::HttpGenerationRecord>,
    verified_mirrors: HashMap<String, observation::VerifiedMirrorRecord>,
    next_http_generation: u64,
    quarantined: bool,
    binding: RepresentationBinding,
    timeline: Option<MediaTimeline>,
    tail_timeline_needed: bool,
    preview: Option<PreviewDescriptor>,
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
            binding: RepresentationBinding::new(post.clone(), &meta, generation, None),
            post,
            renditions: renditions::RenditionState::new(meta.clone(), variants),
            meta,
            evidence: HashMap::new(),
            ledger: crate::evidence::EvidenceLedger::default(),
            evidence_clock_ms: 0,
            http_clocks: HashMap::new(),
            http_generations: HashMap::new(),
            verified_mirrors: HashMap::new(),
            next_http_generation: 1,
            quarantined: false,
            timeline: None,
            tail_timeline_needed: false,
            preview: None,
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
        self.switch(meta, generation, None);
    }

    fn selected_meta(
        &self,
        input: RenditionSelection,
        excluded: &HashSet<crate::representation::RepresentationId>,
    ) -> Option<VideoMeta> {
        self.renditions
            .select(self.binding.representation(), input, excluded)
    }

    pub fn binding(&self) -> RepresentationBinding {
        self.binding.clone()
    }

    pub(super) fn is_quarantined(&self) -> bool {
        self.quarantined
    }

    pub fn timeline(&self) -> Option<&MediaTimeline> {
        self.timeline.as_ref()
    }

    pub const fn preview(&self) -> Option<PreviewDescriptor> {
        self.preview
    }

    fn needs_tail_probe(&self) -> bool {
        self.timeline.is_none() && self.meta.duration_ms.is_none()
    }

    pub(super) fn needs_timeline_probe(&self) -> bool {
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

mod defaults;
impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&PostId) -> bool) {
        let removed: Vec<_> = self
            .entries
            .iter()
            .filter(|(post, _)| !keep(post))
            .map(|(post, entry)| {
                (
                    post.clone(),
                    entry.meta.sha256.clone(),
                    entry.renditions.advertised_digest().map(str::to_owned),
                )
            })
            .collect();
        for (post, active, advertised) in removed {
            if active.as_deref().map(str::to_ascii_lowercase)
                != advertised.as_deref().map(str::to_ascii_lowercase)
            {
                self.update_digest_claim(&post, active.as_deref(), advertised.as_deref());
            }
            self.entries.remove(&post);
        }
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

    pub fn set_preview(&mut self, post: &PostId, preview: Option<PreviewDescriptor>) {
        if let Some(entry) = self.entries.get_mut(post) {
            entry.preview = preview;
        }
    }
}

mod bitrate;
mod renditions;
mod representation;
pub use renditions::RenditionQualityEvidence;
mod timeline;

#[cfg(test)]
#[path = "catalog_axiom_test.rs"]
mod axiom_test_support;
