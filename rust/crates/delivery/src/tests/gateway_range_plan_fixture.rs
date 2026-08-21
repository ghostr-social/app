use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::{planned_work, PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_fixture::playback_for;
use crate::tests::media_timeline_fixture::install_classic_timeline;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::catalog::LearnedFacts;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::{ByteRange, DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub(super) fn demand_plan(demanded: ByteRange) -> PlannedWork {
    build_demand_plan(demanded, false)
}

pub(super) fn buffered_demand_plan(demanded: ByteRange) -> PlannedWork {
    build_demand_plan(demanded, true)
}

fn build_demand_plan(demanded: ByteRange, buffered: bool) -> PlannedWork {
    let post = PostId::new("current");
    let mut state = state(post.clone());
    if buffered {
        state.apply_playback(playback_for(post.clone(), 10_000));
    }
    let stats = stats(buffered);
    let retry = RetryBook::new(RetryPolicy::default());
    let demanded = HashMap::from([(post.clone(), demanded)]);
    let independent_sources = HashMap::new();
    let completed_head_probes = HashSet::new();
    let revisions = HashMap::new();
    planned_work(
        &mut state,
        PlanInputs {
            stats: &stats,
            retry: &retry,
            present: &HashMap::new(),
            finalized: &HashSet::new(),
            stored_totals: &HashMap::new(),
            continuation_sources: &HashMap::new(),
            revisions: &revisions,
            independent_sources: &independent_sources,
            completed_head_probes: &completed_head_probes,
            in_flight: &[],
            active_head_probes: &[],
            storage: StorageSnapshot::new(2_000_000_000, 0),
            connection_capacity: 1,
            connection_ceiling: 1,
            per_authority_request_limit: 1,
            packet_loss_bps: 0,
            measured_network_bytes_per_second: 0,
            capacity_revision: 0,
            observed_at_ms: 1,
            demanded: &demanded,
        },
    )
}

fn stats(buffered: bool) -> HostStats {
    let mut stats = HostStats::new();
    if buffered {
        let sample = ThroughputSample::new(1_000_000, Duration::from_secs(1), 1_000, 1).unwrap();
        stats.record_overall_throughput(sample);
        stats.record_host_throughput("media.example", sample);
    }
    stats
}

fn state(post: PostId) -> DeliveryState {
    let meta = VideoMeta {
        urls: vec!["https://media.example/video.mp4".into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(20_000),
        duration_ms: Some(1_000),
    };
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(
            vec![FocusItem {
                post: post.clone(),
                meta,
            }],
            0,
            0,
        ),
        0,
    );
    state.catalog_mut().learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..LearnedFacts::default()
        },
    );
    install_classic_timeline(&mut state, &post, 100, 100);
    state
}
