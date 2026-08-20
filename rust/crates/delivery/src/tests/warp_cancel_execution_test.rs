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
    let post = PostId::new("p11");
    let identity = state
        .catalog()
        .transfer_identity(&post, &source(11))
        .unwrap();
    let active = ActiveAction::range(
        ChunkId {
            post,
            range: ByteRange::new(0, 100_000),
        },
        identity,
        0,
    );
    let work = run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(0, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[active],
        connection_capacity: 1,
    });

    assert_eq!(
        reconcile_warp::execution(work).directive,
        WarpDirective::Cancel(ActionId::new(1))
    );
}
