use super::{
    default_limits, HeadProbeHistory, PlannerCandidateContext, PlannerCapability, PlannerContext,
    PlannerQuality, PlannerRetryAvailability, PlannerWatchEvidence, PreviewAvailability,
    RequestOccupancy, SemanticScore, TwinEpochs,
};
use crate::adaptive::{EpsilonBuckets, PlayabilitySnapshot};
use std::collections::BTreeMap;

pub(super) fn build(snapshot: &PlayabilitySnapshot) -> PlannerContext {
    let mut ranked: Vec<_> = snapshot
        .candidates
        .iter()
        .map(|item| (item.feed_offset.value(), item.post.clone()))
        .chain(
            snapshot
                .hls_candidates
                .iter()
                .map(|item| (item.feed_offset.value(), item.post.clone())),
        )
        .collect();
    ranked.sort_by_key(|item| item.0);
    let candidates = ranked
        .into_iter()
        .enumerate()
        .map(|(rank, (_, post))| (post, unavailable(rank)))
        .collect();
    let progressive = snapshot
        .candidates
        .iter()
        .flat_map(|candidate| &candidate.in_flight)
        .map(|active| active.source.as_str());
    let hls = snapshot
        .hls_candidates
        .iter()
        .filter_map(|candidate| candidate.active_source());
    PlannerContext {
        candidates,
        active: BTreeMap::new(),
        network_class: crate::origin_model::NetworkClass::Unavailable,
        segmented_storage_available_bytes: Default::default(),
        limits: default_limits(snapshot),
        request_occupancy: RequestOccupancy::from_sources(progressive.chain(hls)),
        request_scope: None,
        feedback: None,
        epochs: TwinEpochs::new(0, 0, 0),
        epsilon: EpsilonBuckets::new(20, snapshot.request_slice_bytes, 100, 100),
    }
}

fn unavailable(rank: usize) -> PlannerCandidateContext {
    PlannerCandidateContext {
        semantic: SemanticScore::Unavailable { rank },
        capability: PlannerCapability::Unavailable,
        quality: PlannerQuality::Unavailable,
        preview: PreviewAvailability::Unavailable,
        watch: PlannerWatchEvidence::Unavailable,
        head_probe: HeadProbeHistory::Unobserved,
        retry: PlannerRetryAvailability::Ready,
        whole_body_exhaustion: None,
    }
}
