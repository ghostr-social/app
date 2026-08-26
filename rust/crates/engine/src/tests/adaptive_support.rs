use crate::adaptive::{
    AllocationPlan, CandidateSnapshot, FeedOffset, MediaLayout, NavigationSnapshot,
    NetworkSnapshot, OriginHealth, PlayabilitySnapshot, PlaybackSnapshot, StorageSnapshot,
    ViewProbability,
};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::tests::support::{playable_range, set_reliable_total_bytes};
use crate::PostId;
use std::collections::HashSet;

const MIB: u64 = 1024 * 1024;
const DEFAULT_ORIGIN: &str = "https://origin.example/media";
pub(crate) fn snapshot(
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
            per_authority_request_limit: 6,
            confidence: EstimateConfidence::High,
        },
        storage: StorageSnapshot::new(2 * 1024 * MIB, 0),
        navigation: NavigationSnapshot {
            forward_swipes_per_minute,
            backward_swipes_per_minute: 0,
        },
        candidates: (0..candidate_count).map(candidate).collect(),
        hls_candidates: Vec::new(),
    }
}

fn candidate(distance: usize) -> CandidateSnapshot {
    let playable_ranges: Vec<_> = (0..15).map(playable_range).collect();
    let mut candidate = CandidateSnapshot {
        post: PostId::new(format!("p{distance}")),
        feed_offset: FeedOffset::new(distance as i32),
        view_probability: ViewProbability::new(0.88_f64.powi(distance as i32))
            .expect("valid test fixture"),
        retrieval_eligible: true,
        total_bytes: None,
        bitrate_bps: 1_000_000,
        duration_ms: 60_000,
        layout: MediaLayout::Streamable,
        preferred_source: None,
        startup: StartupFootprint::new(
            vec![playable_ranges[0].bytes],
            playable_ranges[0].playable_ms,
            StartupProvenance::ClassicMp4V1,
        ),
        player_preparation: crate::adaptive::PlayerPreparation::FirstFrameRendered,
        direct_playback_blocked: false,
        timeline_probe: None,
        playable_ranges,
        demanded: None,
        present: Vec::new(),
        finalized: false,
        recently_evicted: Vec::new(),
        in_flight: Vec::new(),
        origins: vec![healthy_origin(DEFAULT_ORIGIN, 20_000_000, 50)],
        evidence: Default::default(),
    };
    set_reliable_total_bytes(&mut candidate, 3_750_000, 10_000);
    candidate
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
