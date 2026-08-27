use crate::manager::plan::axiom_test_support::planned_work;
use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::PlanInputs;
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::{ByteRange, DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::{HashMap, HashSet};
use core::time::Duration;

#[test]
fn stored_mirror_generation_controls_continuation_source_and_extent() {
    let post = PostId::new("post");
    let mut state = state(post.clone());
    let mirror = "https://mirror.test/video.mp4";
    let identity = state.catalog().transfer_identity(&post, mirror).expect("valid test fixture");
    state.catalog_mut().learn_response_for(
        &identity,
        LearnedFacts {
            content_length: Some(16),
            accept_ranges: Some(true),
            host: None,
        },
    );
    let present = HashMap::from([(post.clone(), vec![ByteRange::new(0, 8)])]);
    let stored_totals = HashMap::from([(post.clone(), 16)]);
    let continuation_sources = HashMap::from([(post.clone(), mirror.to_owned())]);
    let independent_sources = HashMap::new();
    let whole_body_exhaustions = HashMap::new();
    let completed_head_probes = HashSet::new();
    let revisions = HashMap::new();
    let finalized = HashSet::new();
    let stats = stats();
    let retry = RetryBook::new(RetryPolicy::default());
    let demanded = HashMap::new();
    let work = planned_work(
        &state,
        &PlanInputs {
            stats: &stats,
            retry: &retry,
            present: &present,
            finalized: &finalized,
            stored_totals: &stored_totals,
            continuation_sources: &continuation_sources,
            revisions: &revisions,
            independent_sources: &independent_sources,
            whole_body_exhaustions: &whole_body_exhaustions,
            completed_head_probes: &completed_head_probes,
            in_flight: &[],
            active_head_probes: &[],
            hls_candidates: &[],
            active_hls_sources: &[],
            segmented_storage_available_bytes: u64::MAX,
            storage: StorageSnapshot::new(1_000_000, 8),
            connection_capacity: 1,
            hls_demand_expansion_allowed: true,
            connection_ceiling: 1,
            per_authority_request_limit: 1,
            packet_loss_bps: 0,
            resource_feedback: None,
            capacity_revision: 0,
            observed_at_ms: 1,
            demanded: &demanded,
        },
    );

    let transfer = work.plan.allocations.first().expect("mirror continuation");
    assert_eq!(transfer.source, mirror);
    assert_eq!(transfer.request.requested_bytes(), ByteRange::new(8, 16));
}

fn state(post: PostId) -> DeliveryState {
    let meta = VideoMeta {
        urls: vec![
            "https://primary.test/video.mp4".to_owned(),
            "https://mirror.test/video.mp4".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(vec![FocusItem { post, meta }], 0, 0),
        0,
    );
    state
}

fn stats() -> HostStats {
    let mut stats = HostStats::new();
    let sample = ThroughputSample::new(1_000_000, Duration::from_secs(1), 1, 1).expect("valid test fixture");
    stats.record_overall_throughput(sample);
    stats
}
