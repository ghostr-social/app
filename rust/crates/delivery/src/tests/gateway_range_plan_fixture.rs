use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::axiom_test_support::planned_work;
use crate::manager::plan::{PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_fixture::playback_for;
use crate::tests::media_timeline_fixture::install_classic_timeline;
use core::time::Duration;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::{ByteRange, DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::{HashMap, HashSet};

pub(super) const OBJECT_BYTES: u64 = 20_000;

pub(super) fn demand_plan(demanded: ByteRange) -> PlannedWork {
    build_demand_plan(demanded, false, OBJECT_BYTES)
}
pub(super) fn buffered_demand_plan(demanded: ByteRange) -> PlannedWork {
    build_demand_plan(demanded, true, OBJECT_BYTES)
}
pub(super) fn sized_demand_plan(demanded: ByteRange, size_bytes: u64) -> PlannedWork {
    build_demand_plan(demanded, false, size_bytes)
}
fn build_demand_plan(demanded: ByteRange, buffered: bool, size_bytes: u64) -> PlannedWork {
    let post = PostId::new("current");
    let mut state = state(&post, size_bytes);
    if buffered {
        state.apply_playback(&playback_for(post.clone(), 10_000));
    }
    let retry = RetryBook::new(RetryPolicy::default());
    planned_work(
        &state,
        &PlanInputs {
            stats: &stats(buffered),
            retry: &retry,
            present: &HashMap::new(),
            finalized: &HashSet::new(),
            stored_totals: &HashMap::new(),
            continuation_sources: &HashMap::new(),
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
            storage: StorageSnapshot::new(2_000_000_000, 0),
            connection_capacity: 1,
            hls_demand_expansion_allowed: true,
            connection_ceiling: 1,
            per_authority_request_limit: 1,
            packet_loss_bps: 0,
            resource_feedback: None,
            capacity_revision: 0,
            observed_at_ms: 1,
            demanded: &HashMap::from([(post.clone(), demanded)]),
        },
    )
}

fn stats(buffered: bool) -> HostStats {
    let mut stats = HostStats::new();
    if buffered {
        let sample =
            ThroughputSample::new(1_000_000, Duration::from_secs(1), 1_000, 1).expect("fixture");
        stats.record_overall_throughput(sample);
        stats.record_host_throughput("media.example", sample);
    }
    stats
}

fn state(post: &PostId, size_bytes: u64) -> DeliveryState {
    let meta = VideoMeta {
        urls: vec!["https://media.example/video.mp4".into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(size_bytes),
        duration_ms: Some(1_000),
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let item = FocusItem {
        post: post.clone(),
        meta,
    };
    state.apply_focus(DeliveryFocus::compatibility(vec![item], 0, 0), 0);
    state.catalog_mut().learn(
        post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    install_classic_timeline(&mut state, post, 100, 100);
    state
}
