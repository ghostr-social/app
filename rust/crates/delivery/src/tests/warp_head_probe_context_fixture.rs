use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::plan::{planned_work, PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{PlannerCommand, StorageSnapshot};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::{HashMap, HashSet};

pub(super) fn generates_head(work: PlannedWork) -> bool {
    work.warp.unwrap().generated.actions.iter().any(|action| {
        matches!(&action.command, PlannerCommand::ProbeHead { post, .. } if post.as_str() == "post")
    })
}

pub(super) fn plan(
    state: &mut DeliveryState,
    active: &[TransferIdentity],
    capacity: usize,
) -> PlannedWork {
    plan_at(state, active, &HashSet::new(), 1, capacity)
}

pub(super) fn plan_at(
    state: &mut DeliveryState,
    active: &[TransferIdentity],
    completed: &HashSet<TransferIdentity>,
    observed_at_ms: u64,
    capacity: usize,
) -> PlannedWork {
    planned_work(
        state,
        PlanInputs {
            stats: &HostStats::new(),
            retry: &RetryBook::new(RetryPolicy::default()),
            present: &HashMap::new(),
            finalized: &HashSet::new(),
            stored_totals: &HashMap::new(),
            continuation_sources: &HashMap::new(),
            revisions: &HashMap::new(),
            independent_sources: &HashMap::new(),
            whole_body_exhaustions: &HashMap::new(),
            completed_head_probes: completed,
            in_flight: &[],
            active_head_probes: active,
            hls_candidates: &[],
            active_hls_sources: &[],
            segmented_storage_available_bytes: u64::MAX,
            storage: StorageSnapshot::new(1_000_000, 0),
            connection_capacity: capacity,
            hls_demand_expansion_allowed: true,
            connection_ceiling: 3,
            per_authority_request_limit: 3,
            packet_loss_bps: 0,
            resource_feedback: None,
            capacity_revision: 0,
            observed_at_ms,
            demanded: &HashMap::new(),
        },
    )
}

pub(super) fn state(post: PostId, source: &str) -> DeliveryState {
    state_with_sources(post, vec![source.to_owned()])
}

pub(super) fn state_with_sources(post: PostId, sources: Vec<String>) -> DeliveryState {
    state_with_meta(post, sources, None)
}

pub(super) fn state_with_size(post: PostId, source: &str, size: u64) -> DeliveryState {
    state_with_meta(post, vec![source.to_owned()], Some(size))
}

fn state_with_meta(post: PostId, sources: Vec<String>, size_bytes: Option<u64>) -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let item = FocusItem {
        post,
        meta: VideoMeta {
            urls: sources,
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes,
            duration_ms: None,
        },
    };
    state.apply_focus(DeliveryFocus::compatibility(vec![item], 0, 0), 0);
    state
}
