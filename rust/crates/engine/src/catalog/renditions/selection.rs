use super::Catalog;
use crate::playback::{BufferTarget, NetworkConditions, PlaybackObservation};
use crate::representation::{RepresentationBinding, RepresentationId};
use crate::{PostId, VideoMeta};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub struct RenditionSelection {
    pub(super) network: NetworkConditions,
    pub(super) observation: PlaybackObservation,
    pub(super) target: BufferTarget,
}

impl RenditionSelection {
    pub fn new(
        network: NetworkConditions,
        observation: PlaybackObservation,
        target: BufferTarget,
    ) -> Self {
        Self {
            network,
            observation,
            target,
        }
    }
}

impl Catalog {
    pub fn rendition_variants(
        &self,
        post: &PostId,
    ) -> Option<&[crate::video_rendition::VideoRendition]> {
        Some(self.entries.get(post)?.renditions.variants())
    }

    pub fn select_rendition_by_representation(
        &mut self,
        post: &PostId,
        representation: &RepresentationId,
    ) -> Option<RepresentationBinding> {
        let selected = self
            .entries
            .get(post)?
            .renditions
            .meta_for(representation)?;
        self.switch_rendition(post, selected)
    }

    pub fn select_rendition_excluding(
        &mut self,
        post: &PostId,
        input: RenditionSelection,
        excluded: &HashSet<RepresentationId>,
    ) -> Option<RepresentationBinding> {
        let selected = self.entries.get(post)?.selected_meta(input, excluded)?;
        self.switch_rendition(post, selected)
    }

    fn switch_rendition(
        &mut self,
        post: &PostId,
        selected: VideoMeta,
    ) -> Option<RepresentationBinding> {
        if self.quarantined(&selected) {
            return None;
        }
        let entry = self.entries.get(post)?;
        let source = entry.binding.source_representation().clone();
        let previous_digest = entry.meta.sha256.clone();
        let next_digest = selected.sha256.clone();
        let generation = self.allocate_generation();
        self.entries
            .get_mut(post)?
            .switch(selected, generation, Some(source));
        self.update_digest_claim(post, previous_digest.as_deref(), next_digest.as_deref());
        self.apply_known_quarantine(post);
        self.binding(post)
    }

    fn quarantined(&self, selected: &VideoMeta) -> bool {
        selected.sha256.as_deref().is_some_and(|digest| {
            self.quarantined_digests
                .contains(&digest.to_ascii_lowercase())
        })
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "selection/test_support.rs"]
mod test_support;
