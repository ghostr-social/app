use crate::manager::inflight::ActiveAction;
use crate::manager::reconcile_warp::{self, WarpDirective};
use crate::tests::adaptive_plan_fixture::{source, state};
use crate::tests::adaptive_plan_runner::{run, PlanScenario};
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::{ActionId, ByteRange, ChunkId, PostId};
use std::collections::HashMap;

#[test]
fn selected_cancel_targets_the_exact_obsolete_action() {
    let state = state();
    let active = [active(&state, 10, 1), active(&state, 11, 2)];
    let work = run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(0, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &active,
        connection_capacity: 1,
    });
    let execution = reconcile_warp::execution(work);
    let WarpDirective::Cancel(selected) = execution.directive else {
        panic!("one exact cancel must be selected");
    };
    let other = if selected == ActionId::new(1) {
        ActionId::new(2)
    } else {
        ActionId::new(1)
    };

    assert!([ActionId::new(1), ActionId::new(2)].contains(&selected));
    assert!(!execution.retained.contains(&selected));
    assert!(execution.retained.contains(&other));
}

fn active(state: &crate::manager::state::DeliveryState, index: usize, id: u64) -> ActiveAction {
    let post = PostId::new(format!("p{index}"));
    let identity = state
        .catalog()
        .transfer_identity(&post, &source(index))
        .expect("valid test fixture");
    ActiveAction::range_with_action(
        ActionId::new(id),
        ChunkId {
            post,
            range: ByteRange::new(0, 100_000),
        },
        identity,
        0,
    )
}
