use crate::adaptive::{
    AllocationPlan, CandidateSnapshot, FeedOffset, MediaLayout, NavigationSnapshot,
    NetworkSnapshot, OriginHealth, PlayabilitySnapshot, PlayableRange, PlaybackSnapshot,
    StorageSnapshot, ViewProbability,
};
use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::{ByteRange, PostId};
use std::collections::HashSet;

const MIB: u64 = 1024 * 1024;

pub(super) fn snapshot(
    candidate_count: usize,
    throughput_bps: u64,
    buffer_ms: u64,
    forward_swipes_per_minute: u16,
) -> PlayabilitySnapshot {
    PlayabilitySnapshot {
        observed_at_ms: 10_000,
        commitment_ms: 3_000,
        request_slice_bytes: crate::adaptive::REQUEST_SLICE_BYTES,
        playback: PlaybackSnapshot {
            current: PostId::new("p0"),
            authority: crate::adaptive::CurrentAuthority::Canonical,
            phase: PlaybackPhase::Playing,
            buffer_ahead_ms: buffer_ms,
        },
        network: NetworkSnapshot {
            throughput_bps,
            rtt_ms: 50,
            packet_loss_bps: 0,
            connection_capacity: 6,
            connection_ceiling: 6,
            confidence: EstimateConfidence::High,
        },
        storage: StorageSnapshot::new(2 * 1024 * MIB, 0),
        navigation: NavigationSnapshot {
            forward_swipes_per_minute,
            backward_swipes_per_minute: 0,
        },
        candidates: (0..candidate_count).map(candidate).collect(),
    }
}

fn candidate(distance: usize) -> CandidateSnapshot {
    CandidateSnapshot {
        post: PostId::new(format!("p{distance}")),
        feed_offset: FeedOffset::new(distance as i32),
        view_probability: ViewProbability::new(0.88_f64.powi(distance as i32)).unwrap(),
        total_bytes: Some(3_750_000),
        bitrate_bps: 1_000_000,
        duration_ms: 60_000,
        layout: MediaLayout::Streamable,
        timeline_probe: None,
        playable_ranges: (0..15).map(playable_range).collect(),
        demanded: None,
        present: Vec::new(),
        recently_evicted: Vec::new(),
        in_flight: Vec::new(),
        origins: vec![healthy_origin("origin", 20_000_000, 50)],
    }
}

pub(super) fn healthy_origin(source: &str, throughput_bps: u64, rtt_ms: u64) -> OriginHealth {
    OriginHealth {
        source: source.to_owned(),
        available: true,
        throughput_bps,
        rtt_ms,
        packet_loss_bps: 0,
        failure_bps: 0,
    }
}

pub(super) fn frontier(plan: &AllocationPlan) -> Vec<PostId> {
    let mut seen = HashSet::new();
    plan.allocations
        .iter()
        .map(|work| work.post.clone())
        .chain(plan.retained.iter().map(|work| work.post.clone()))
        .filter(|post| seen.insert(post.clone()))
        .collect()
}

pub(super) fn planned_playable_ms(plan: &AllocationPlan, post: &PostId) -> u64 {
    plan.allocations
        .iter()
        .filter(|work| &work.post == post)
        .map(|work| work.expected_playable_gain_ms)
        .sum()
}

fn playable_range(index: u64) -> PlayableRange {
    PlayableRange {
        bytes: ByteRange::new(index * 250_000, (index + 1) * 250_000),
        playable_ms: 2_000,
    }
}
