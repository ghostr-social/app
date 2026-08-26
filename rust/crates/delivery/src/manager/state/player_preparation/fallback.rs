use super::evidence::{capability_profile, rendition_capability_profile};
use super::DeliveryState;
use crate::client_capability::ClientCapabilityStatus;
use ghostr_engine::representation::{RepresentationBinding, RepresentationId};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::PostId;
use std::collections::HashSet;

impl DeliveryState {
    pub(crate) fn select_capability_fallback(
        &mut self,
        post: &PostId,
        now_ms: u64,
    ) -> Option<RepresentationBinding> {
        let generation = self.client_capabilities.current_generation()?;
        let active = self.catalog.binding(post)?.representation().clone();
        if self.client_capability_status(post, generation, now_ms)
            != ClientCapabilityStatus::Unsupported
        {
            return None;
        }
        let selected = self.best_fallback(post, generation, &active, now_ms)?;
        self.catalog
            .select_rendition_by_representation(post, &selected)
    }

    pub(crate) fn select_known_capability_fallbacks(
        &mut self,
        now_ms: u64,
    ) -> Vec<RepresentationBinding> {
        self.planning_window_posts()
            .into_iter()
            .filter_map(|post| self.select_capability_fallback(&post, now_ms))
            .collect()
    }

    pub(crate) fn decoder_blocked_representations(
        &self,
        post: &PostId,
        now_ms: u64,
    ) -> HashSet<RepresentationId> {
        let Some(generation) = self.client_capabilities.current_generation() else {
            return HashSet::new();
        };
        self.catalog
            .rendition_variants(post)
            .into_iter()
            .flatten()
            .filter(|variant| {
                unavailable(self.rendition_capability_status(post, variant, generation, now_ms))
            })
            .map(|variant| variant.identity())
            .collect()
    }

    fn best_fallback(
        &self,
        post: &PostId,
        generation: u64,
        active: &RepresentationId,
        now_ms: u64,
    ) -> Option<RepresentationId> {
        self.catalog
            .rendition_variants(post)?
            .iter()
            .filter(|variant| variant.identity() != *active)
            .filter_map(|variant| {
                let identity = variant.identity();
                let status = self.rendition_capability_status(post, variant, generation, now_ms);
                fallback_rank(status).map(|rank| (rank, variant, identity))
            })
            .min_by_key(|(rank, variant, _)| {
                (*rank, variant.bitrate_bits_per_second().unwrap_or(u64::MAX))
            })
            .map(|(_, _, identity)| identity)
    }

    fn rendition_capability_status(
        &self,
        post: &PostId,
        variant: &VideoRendition,
        generation: u64,
        now_ms: u64,
    ) -> ClientCapabilityStatus {
        let active = self.catalog.binding(post);
        let profile = if active
            .as_ref()
            .is_some_and(|binding| binding.representation() == &variant.identity())
        {
            let binding = self.catalog.binding(post);
            binding.and_then(|binding| capability_profile(&self.catalog, post, &binding, now_ms))
        } else {
            rendition_capability_profile(post, variant)
        };
        profile.map_or(ClientCapabilityStatus::Unknown, |profile| {
            self.client_capabilities
                .content_status(generation, &profile)
        })
    }
}

fn unavailable(status: ClientCapabilityStatus) -> bool {
    matches!(
        status,
        ClientCapabilityStatus::Testing
            | ClientCapabilityStatus::Unsupported
            | ClientCapabilityStatus::Inconclusive
    )
}

fn fallback_rank(status: ClientCapabilityStatus) -> Option<u8> {
    match status {
        ClientCapabilityStatus::Supported { .. } => Some(0),
        ClientCapabilityStatus::Unknown => Some(1),
        ClientCapabilityStatus::Testing
        | ClientCapabilityStatus::Unsupported
        | ClientCapabilityStatus::Inconclusive => None,
    }
}
