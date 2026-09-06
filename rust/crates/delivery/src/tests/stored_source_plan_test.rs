use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::axiom_test_support::planned_work;
use crate::manager::plan::PlanInputs;
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use core::time::Duration;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::{ByteRange, DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::{HashMap, HashSet};

#[test]
fn stored_mirror_generation_controls_continuation_source_and_extent() {
    let post = PostId::new("post");
    let mut state = state(post.clone());
    let mirror = "https://mirror.test/video.mp4";
    let identity = state
        .catalog()
        .transfer_identity(&post, mirror)
        .expect("valid test fixture");
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
    let stats = stats();
    let retry = RetryBook::new(RetryPolicy::default());
    let work = planned_work(
        &state,
        &PlanInputs {
            stats: &stats,
            retry: &retry,
            present: &present,
            finalized: &HashSet::new(),
            stored_totals: &stored_totals,
            continuation_sources: &continuation_sources,
            revisions: &HashMap::new(),
            independent_sources: &HashMap::new(),
            whole_body_exhaustions: &HashMap::new(),
            completed_head_probes: &HashSet::new(),
            unavailable_head_probes: &HashSet::new(),
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
            current_watch: Duration::ZERO,
            watch_window: &crate::qoe::WatchWindow::default(),
            observed_at_ms: 1,
            demanded: &HashMap::new(),
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
    let sample =
        ThroughputSample::new(1_000_000, Duration::from_secs(1), 1, 1).expect("valid test fixture");
    stats.record_overall_throughput(sample);
    stats
}
