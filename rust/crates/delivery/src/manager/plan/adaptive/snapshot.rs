use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    candidate_snapshot, CandidateEvidence, FeedOffset, NavigationSnapshot, PlayabilitySnapshot,
    PlayableRange, PlaybackSnapshot,
};
use ghostr_engine::{ByteRange, PostId};

pub(super) fn build(state: &DeliveryState, inputs: &PlanInputs<'_>) -> Option<PlayabilitySnapshot> {
    let current = state.focus().current()?.clone();
    let navigation = state.navigation(inputs.observed_at_ms);
    let network = super::telemetry::network(inputs);
    let candidates = candidates(state, inputs, navigation, &current);
    let playback = playback_snapshot(state, current);
    Some(PlayabilitySnapshot {
        observed_at_ms: inputs.observed_at_ms,
        commitment_ms: state.params().commitment_ms,
        playback,
        network,
        storage: inputs.storage,
        navigation,
        candidates,
    })
}

fn candidates(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    navigation: NavigationSnapshot,
    current: &PostId,
) -> Vec<ghostr_engine::adaptive::CandidateSnapshot> {
    positioned_posts(state, current)
        .into_iter()
        .filter_map(|position| candidate(state, inputs, position, navigation))
        .collect()
}

struct CandidatePosition {
    post: PostId,
    offset: FeedOffset,
}

fn candidate(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    position: CandidatePosition,
    navigation: NavigationSnapshot,
) -> Option<ghostr_engine::adaptive::CandidateSnapshot> {
    let post = position.post;
    let evidence = CandidateEvidence {
        post: post.clone(),
        feed_distance: position.offset.magnitude() as usize,
        view_probability: navigation.view_probability(position.offset),
        present: inputs.present.get(&post).cloned().unwrap_or_default(),
        recently_evicted: state.recently_evicted(&post),
        in_flight: in_flight(state, inputs, &post),
        origins: super::telemetry::origins(state, inputs, &post),
    };
    let mut candidate = candidate_snapshot(state.catalog(), state.params(), evidence)?;
    if let Some(range) = demanded_range(inputs, &post) {
        prioritize_range(&mut candidate.playable_ranges, range, candidate.bitrate_bps);
    }
    Some(candidate)
}

fn in_flight(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    post: &PostId,
) -> Vec<ghostr_engine::adaptive::InFlightRange> {
    inputs
        .in_flight
        .iter()
        .filter(|active| &active.chunk().post == post)
        .map(|active| active_range(state, active))
        .collect()
}

fn active_range(
    state: &DeliveryState,
    active: &crate::manager::inflight::ActiveRange,
) -> ghostr_engine::adaptive::InFlightRange {
    let source = active.identity().source().as_str();
    let current = state
        .catalog()
        .transfer_identity(&active.chunk().post, source);
    ghostr_engine::adaptive::InFlightRange {
        bytes: active.chunk().range,
        source: source.to_owned(),
        committed_until_ms: active.committed_until_ms(),
        identity_current: current.as_ref() == Some(active.identity()),
    }
}

fn positioned_posts(state: &DeliveryState, current: &PostId) -> Vec<CandidatePosition> {
    let posts = state.window_posts();
    let current = posts.iter().position(|post| post == current).unwrap_or(0);
    posts
        .into_iter()
        .enumerate()
        .map(|(index, post)| CandidatePosition {
            post,
            offset: feed_offset(index, current),
        })
        .collect()
}

fn feed_offset(index: usize, current: usize) -> FeedOffset {
    let difference = index as i128 - current as i128;
    FeedOffset::new(difference.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32)
}

fn playback_snapshot(state: &DeliveryState, current: PostId) -> PlaybackSnapshot {
    let observation = state.playback().observation();
    PlaybackSnapshot {
        current,
        phase: observation.map_or(ghostr_engine::playback::PlaybackPhase::Starting, |value| {
            value.phase()
        }),
        buffer_ahead_ms: observation.map_or(0, |value| value.buffer_ahead().as_millis() as u64),
    }
}

fn demanded_range(inputs: &PlanInputs<'_>, post: &PostId) -> Option<ByteRange> {
    inputs
        .demanded
        .as_ref()
        .filter(|signal| &signal.post == post)
        .map(|signal| signal.range)
}

fn prioritize_range(ranges: &mut Vec<PlayableRange>, wanted: ByteRange, bitrate_bps: u64) {
    let mut surrounding = Vec::new();
    let mut overlap_gain = 0_u64;
    for playable in std::mem::take(ranges) {
        let (pieces, gain) = split_around(playable, wanted);
        surrounding.extend(pieces);
        overlap_gain = overlap_gain.saturating_add(gain);
    }
    ranges.push(PlayableRange {
        bytes: wanted,
        playable_ms: demanded_gain(wanted, bitrate_bps, overlap_gain),
    });
    ranges.extend(surrounding);
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}

fn split_around(playable: PlayableRange, wanted: ByteRange) -> (Vec<PlayableRange>, u64) {
    if !overlaps(playable.bytes, wanted) {
        return (vec![playable], 0);
    }
    let overlap = ByteRange::new(
        playable.bytes.start.max(wanted.start),
        playable.bytes.end.min(wanted.end),
    );
    let pieces = [
        piece(playable, playable.bytes.start, overlap.start),
        piece(playable, overlap.end, playable.bytes.end),
    ]
    .into_iter()
    .flatten()
    .collect();
    (pieces, proportional_gain(playable, overlap.len()))
}

fn piece(playable: PlayableRange, start: u64, end: u64) -> Option<PlayableRange> {
    (start < end).then(|| PlayableRange {
        bytes: ByteRange::new(start, end),
        playable_ms: proportional_gain(playable, end - start),
    })
}

fn proportional_gain(playable: PlayableRange, bytes: u64) -> u64 {
    let gain = u128::from(playable.playable_ms).saturating_mul(u128::from(bytes));
    (gain / u128::from(playable.bytes.len().max(1)))
        .max(1)
        .min(u128::from(u64::MAX)) as u64
}

fn demanded_gain(wanted: ByteRange, bitrate_bps: u64, overlap_gain: u64) -> u64 {
    if overlap_gain > 0 {
        return overlap_gain;
    }
    wanted
        .len()
        .saturating_mul(8_000)
        .div_ceil(bitrate_bps.max(1))
        .max(1)
}
