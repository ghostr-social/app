use crate::manager::inflight::ActiveAction;
use crate::manager::plan::PlannedWork;
use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_runner::{
    run_with_hls, run_with_hls_retry, run_with_per_authority_limit, PlanScenario,
};
use crate::tests::provisional_handoff_fixture::DetachedFuture;
use ghostr_engine::adaptive::{PlannerCommand, StorageSnapshot};
use ghostr_engine::ActionId;
use std::collections::{HashMap, HashSet};

pub(super) fn plan(
    state: DeliveryState,
    active: &[ActiveAction],
    per_authority_limit: usize,
) -> PlannedWork {
    run_with_per_authority_limit(
        PlanScenario {
            state,
            buffer_ms: 0,
            bytes_per_second: 4_000_000,
            storage: StorageSnapshot::new(2_000_000_000, 0),
            present: HashMap::new(),
            packet_loss_bps: 0,
            in_flight: active,
            connection_capacity: 1,
        },
        per_authority_limit,
    )
}

pub(super) fn plan_detached(fixture: DetachedFuture) -> PlannedWork {
    let DetachedFuture { state, active } = fixture;
    plan(state, &[active], 3)
}

pub(super) fn plan_hls(
    state: DeliveryState,
    active: &[ActiveAction],
    hls: &[ghostr_engine::adaptive::HlsCandidateSnapshot],
) -> PlannedWork {
    run_with_hls(scenario(state, active), hls, 2)
}

pub(super) fn plan_hls_with_retry(
    state: DeliveryState,
    active: &[ActiveAction],
    hls: &[ghostr_engine::adaptive::HlsCandidateSnapshot],
    retry: &crate::manager::retry::RetryBook,
) -> PlannedWork {
    run_with_hls_retry(scenario(state, active), hls, 2, retry)
}

fn scenario(state: DeliveryState, active: &[ActiveAction]) -> PlanScenario<'_> {
    PlanScenario {
        state,
        buffer_ms: 0,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: active,
        connection_capacity: 1,
    }
}

pub(super) fn generated_cancels(work: &PlannedWork) -> HashSet<ActionId> {
    work.warp
        .iter()
        .flat_map(|warp| &warp.generated.actions)
        .filter_map(|action| match action.command {
            PlannerCommand::Cancel(id) => Some(id),
            _ => None,
        })
        .collect()
}
