use super::hls_burst_floor;
use ghostr_engine::adaptive::{
    CurrentAuthority, FeedOffset, HlsBootstrapStage, HlsBootstrapState, HlsCandidateSnapshot,
    NavigationSnapshot, NetworkSnapshot, PlayabilitySnapshot, PlaybackSnapshot, StorageSnapshot,
    ViewProbability,
};
use ghostr_engine::playback::{EstimateConfidence, PlaybackPhase};
use ghostr_engine::PostId;

#[test]
fn hls_demand_cannot_mint_network_burst_capacity() {
    let snapshot = PlayabilitySnapshot {
        observed_at_ms: 1,
        commitment_ms: 2_000,
        request_slice_bytes: 256 * 1024,
        playback: PlaybackSnapshot {
            current: PostId::new("hls-0"),
            authority: CurrentAuthority::Canonical,
            phase: PlaybackPhase::Starting,
            buffer_ahead_ms: 0,
        },
        network: NetworkSnapshot {
            throughput_bps: 1_000_000,
            rtt_ms: 50,
            packet_loss_bps: 0,
            connection_capacity: 3,
            connection_ceiling: 3,
            per_authority_request_limit: 2,
            confidence: EstimateConfidence::High,
        },
        storage: StorageSnapshot::new(64 * 1024 * 1024, 0),
        navigation: NavigationSnapshot {
            forward_swipes_per_minute: 4,
            backward_swipes_per_minute: 0,
        },
        candidates: Vec::new(),
        hls_candidates: (0..3).map(candidate).collect(),
    };

    assert_eq!(hls_burst_floor(&snapshot, 3), 0);
}

fn candidate(index: usize) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new(format!("hls-{index}")),
        feed_offset: FeedOffset::new(index as i32),
        view_probability: ViewProbability::new(0.8).unwrap(),
        startup_value_ms: 1_000,
        cursor: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::FirstSegment,
            source: format!("https://{index}.example/segment.m4s"),
        },
    }
}
