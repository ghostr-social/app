use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    candidate_snapshot_at, CandidateEvidence, FeedOffset, NavigationSnapshot, PlayabilitySnapshot,
    PlaybackSnapshot,
};
use ghostr_engine::{ByteRange, PostId};
use std::collections::HashSet;

mod demand;

pub(super) fn build(state: &DeliveryState, inputs: &PlanInputs<'_>) -> Option<PlayabilitySnapshot> {
    let current = state.focus().current()?.clone();
    let navigation = state.navigation(inputs.observed_at_ms);
    let network = super::telemetry::network(inputs);
    let candidates = candidates(state, inputs, navigation, &current);
    let playback = playback_snapshot(state, current);
    Some(PlayabilitySnapshot {
        observed_at_ms: inputs.observed_at_ms,
        commitment_ms: state.params().commitment_ms,
        request_slice_bytes: state
            .params()
            .chunk_bytes
            .min(ghostr_engine::adaptive::REQUEST_SLICE_BYTES),
        playback,
        network,
        storage: inputs.storage,
        navigation,
        candidates,
        hls_candidates: inputs.hls_candidates.to_vec(),
    })
}

fn candidates(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    navigation: NavigationSnapshot,
    current: &PostId,
) -> Vec<ghostr_engine::adaptive::CandidateSnapshot> {
    positioned_posts(state, inputs, current)
        .into_iter()
        .filter_map(|position| candidate(state, inputs, position, navigation))
        .collect()
}

struct CandidatePosition {
    post: PostId,
    offset: FeedOffset,
    retrieval_eligible: bool,
}

fn candidate(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    position: CandidatePosition,
    navigation: NavigationSnapshot,
) -> Option<ghostr_engine::adaptive::CandidateSnapshot> {
    let evidence = candidate_evidence(state, inputs, &position, navigation);
    let post = position.post;
    let mut candidate = candidate_snapshot_at(
        state.catalog(),
        state.params(),
        evidence,
        inputs.observed_at_ms,
    )?;
    candidate.player_preparation =
        state.player_preparation(&post, inputs.revisions.get(&post).copied());
    candidate.retrieval_eligible = position.retrieval_eligible;
    candidate.finalized = inputs.finalized.contains(&post);
    if let Some(range) = demanded_range(inputs, &post) {
        demand::prioritize(&mut candidate, range);
        candidate.demanded = Some(range);
    }
    Some(candidate)
}

fn candidate_evidence(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    position: &CandidatePosition,
    navigation: NavigationSnapshot,
) -> CandidateEvidence {
    let post = &position.post;
    CandidateEvidence {
        post: post.clone(),
        feed_offset: position.offset,
        view_probability: navigation.view_probability(position.offset),
        present: inputs.present.get(post).cloned().unwrap_or_default(),
        stored_total: inputs.stored_totals.get(post).copied(),
        continuation_source: inputs.continuation_sources.get(post).cloned(),
        independent_object_sources: inputs
            .independent_sources
            .get(post)
            .cloned()
            .unwrap_or_default(),
        recently_evicted: state.recently_evicted(post),
        in_flight: in_flight(state, inputs, post),
        origins: super::telemetry::origins(state, inputs, post),
    }
}

fn in_flight(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    post: &PostId,
) -> Vec<ghostr_engine::adaptive::InFlightAction> {
    inputs
        .in_flight
        .iter()
        .filter(|active| active.post() == post)
        .map(|active| active_range(state, active))
        .collect()
}

fn active_range(
    state: &DeliveryState,
    active: &crate::manager::inflight::ActiveAction,
) -> ghostr_engine::adaptive::InFlightAction {
    let source = active.identity().source().as_str();
    let current = state.catalog().transfer_identity(active.post(), source);
    ghostr_engine::adaptive::InFlightAction {
        action_id: active.action_id(),
        request: active.request(),
        effective_bytes: active.effective_bytes(),
        reserved_storage_bytes: active.reserved_storage_bytes(),
        source: source.to_owned(),
        committed_until_ms: active.committed_until_ms(),
        identity_current: current.as_ref() == Some(active.identity()),
        cancelling: active.cancelling(),
    }
}

fn positioned_posts(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    current: &PostId,
) -> Vec<CandidatePosition> {
    let planning: HashSet<_> = state.planning_window_posts().into_iter().collect();
    let posts = state.window_posts();
    let current = posts.iter().position(|post| post == current).unwrap_or(0);
    posts
        .into_iter()
        .enumerate()
        .filter(|(_, post)| planning.contains(post) || stored_for_eviction(inputs, post))
        .map(|(index, post)| CandidatePosition {
            retrieval_eligible: planning.contains(&post),
            post,
            offset: feed_offset(index, current),
        })
        .collect()
}

fn stored_for_eviction(inputs: &PlanInputs<'_>, post: &PostId) -> bool {
    inputs
        .present
        .get(post)
        .is_some_and(|ranges| !ranges.is_empty())
        || inputs.finalized.contains(post)
}

fn feed_offset(index: usize, current: usize) -> FeedOffset {
    let difference = index as i128 - current as i128;
    FeedOffset::new(difference.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32)
}

fn playback_snapshot(state: &DeliveryState, current: PostId) -> PlaybackSnapshot {
    let observation = state.playback().observation();
    PlaybackSnapshot {
        current,
        authority: state.current_authority(),
        phase: observation.map_or(ghostr_engine::playback::PlaybackPhase::Starting, |value| {
            value.phase()
        }),
        buffer_ahead_ms: observation.map_or(0, |value| value.buffer_ahead().as_millis() as u64),
    }
}

fn demanded_range(inputs: &PlanInputs<'_>, post: &PostId) -> Option<ByteRange> {
    inputs.demanded.get(post).copied()
}
