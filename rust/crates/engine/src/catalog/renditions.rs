use super::{Catalog, CatalogEntry};
use crate::evidence::NostrMetadataEvidence;
use crate::rendition::{
    QualitySelectionInput, QualitySelectionPolicy, Rendition, RenditionId, RenditionSet,
};
use crate::representation::{RepresentationBinding, RepresentationId};
use crate::video_rendition::VideoRendition;
use crate::{PostId, VideoMeta};
use std::collections::{BTreeSet, HashSet};

mod integrity;
mod quality;
mod selection;
pub use quality::RenditionQualityEvidence;
pub use selection::RenditionSelection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RenditionState {
    advertised: VideoMeta,
    variants: Vec<VideoRendition>,
}

impl RenditionState {
    pub(super) fn new(advertised: VideoMeta, variants: Vec<VideoRendition>) -> Self {
        Self {
            advertised,
            variants,
        }
    }

    fn matches(&self, meta: &VideoMeta, variants: &[VideoRendition]) -> bool {
        &self.advertised == meta && self.variants == variants
    }

    fn advertised_is(&self, meta: &VideoMeta) -> bool {
        &self.advertised == meta
    }

    pub(super) fn active_bitrate(&self, active: &RepresentationId) -> Option<u64> {
        self.variants
            .iter()
            .find(|variant| variant.identity() == *active)
            .and_then(VideoRendition::bitrate_bits_per_second)
    }

    fn variants(&self) -> &[VideoRendition] {
        &self.variants
    }

    fn meta_for(&self, representation: &RepresentationId) -> Option<VideoMeta> {
        self.variants
            .iter()
            .find(|variant| variant.identity() == *representation)
            .map(|variant| variant.meta().clone())
    }

    pub(super) fn select(
        &self,
        active: &RepresentationId,
        input: RenditionSelection,
        excluded: &HashSet<RepresentationId>,
    ) -> Option<VideoMeta> {
        let (qualities, current) = self.qualities(active, excluded);
        let ladder = RenditionSet::try_new(qualities).ok()?;
        let input =
            QualitySelectionInput::new(input.network, input.observation, input.target, current);
        let selected = QualitySelectionPolicy::default().select(&ladder, &input);
        self.variant(selected.selected().id())
            .filter(|variant| variant.identity() != *active)
            .map(|variant| variant.meta().clone())
    }

    fn qualities(
        &self,
        active: &RepresentationId,
        excluded: &HashSet<RepresentationId>,
    ) -> (Vec<Rendition>, Option<RenditionId>) {
        let current = self.current_quality(active);
        let mut seen = BTreeSet::new();
        let mut qualities = current.clone().into_iter().collect::<Vec<_>>();
        seen.extend(qualities.iter().map(Rendition::bitrate_bits_per_second));
        for quality in self
            .variants
            .iter()
            .filter(|variant| !excluded.contains(&variant.identity()))
            .filter_map(VideoRendition::quality)
        {
            if seen.insert(quality.bitrate_bits_per_second()) {
                qualities.push(quality);
            }
        }
        let current = current.map(|quality| quality.id().clone());
        (qualities, current)
    }

    fn current_quality(&self, active: &RepresentationId) -> Option<Rendition> {
        self.variants
            .iter()
            .find(|variant| variant.identity() == *active)
            .and_then(VideoRendition::quality)
    }

    fn variant(&self, id: &RenditionId) -> Option<&VideoRendition> {
        self.variants
            .iter()
            .find(|variant| variant.quality_id() == *id)
    }
}

impl Catalog {
    pub fn upsert(&mut self, post: PostId, meta: VideoMeta) -> RepresentationBinding {
        if self
            .entries
            .get(&post)
            .is_some_and(|entry| entry.renditions.advertised_is(&meta))
        {
            return self.binding(&post).expect("catalog entry exists");
        }
        self.replace(post, meta, Vec::new())
    }

    pub fn upsert_with_renditions(
        &mut self,
        post: PostId,
        meta: VideoMeta,
        variants: Vec<VideoRendition>,
    ) -> RepresentationBinding {
        if self
            .entries
            .get(&post)
            .is_some_and(|entry| entry.renditions.matches(&meta, &variants))
        {
            return self.binding(&post).expect("catalog entry exists");
        }
        self.replace(post, meta, variants)
    }

    pub fn upsert_with_evidence(
        &mut self,
        post: PostId,
        meta: VideoMeta,
        variants: Vec<VideoRendition>,
        evidence: Vec<NostrMetadataEvidence>,
    ) -> RepresentationBinding {
        let observed_at_ms = evidence
            .iter()
            .map(|item| item.observed_at_ms)
            .max()
            .unwrap_or(0);
        let binding = self.upsert_with_renditions(post, meta, variants);
        if let Some(entry) = self.entries.get_mut(binding.post()) {
            entry.record_nostr_metadata(evidence);
        }
        self.recalibrate(observed_at_ms);
        binding
    }

    fn replace(
        &mut self,
        post: PostId,
        meta: VideoMeta,
        variants: Vec<VideoRendition>,
    ) -> RepresentationBinding {
        let key = post.clone();
        let generation = self.allocate_generation();
        match self.entries.get_mut(&key) {
            Some(entry) => entry.refresh(meta, variants, generation),
            None => {
                self.entries.insert(
                    key.clone(),
                    CatalogEntry::new(post, meta, variants, generation),
                );
            }
        }
        self.apply_known_quarantine(&key);
        self.binding(&key).expect("upserted catalog entry")
    }
}
