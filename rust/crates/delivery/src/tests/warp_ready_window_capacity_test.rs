use crate::manager::inflight::ActiveAction;
use crate::tests::adaptive_plan_fixture::{source, state};
use crate::tests::adaptive_plan_runner::{run, PlanScenario};
use ghostr_engine::adaptive::{PlannerCommand, StorageSnapshot};
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::{HashMap, HashSet};

#[test]
fn ready_window_can_select_new_work_while_the_adaptive_limit_is_one() {
    let state = state();
    let active_post = PostId::new("p1");
    let identity = state
        .catalog()
        .transfer_identity(&active_post, &source(1))
        .expect("valid test fixture");
    let active = ActiveAction::range(
        ChunkId {
            post: active_post.clone(),
            range: ByteRange::new(0, 100_000),
        },
        identity,
        4_000,
    );
    let work = run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[active],
        connection_capacity: 1,
    });
    let base: HashSet<_> = work
        .plan
        .allocations
        .iter()
        .map(|item| item.post.clone())
        .collect();
    let warp = work.warp.expect("valid test fixture");
    let selected = warp.selected.as_ref().expect("new ready-window work");

    assert!(matches!(selected.command, PlannerCommand::Transfer(_)));
    assert_ne!(selected.node.post, active_post);
    assert!(base.contains(&selected.node.post));
    assert_scoped(&warp, &base, &active_post);
}

fn assert_scoped(
    warp: &ghostr_engine::adaptive::WarpPlanningDecision,
    base: &HashSet<PostId>,
    active: &PostId,
) {
    let blocked: Vec<_> = warp
        .generated
        .actions
        .iter()
        .filter(|item| item.node.resources.requests > 0)
        .filter(|item| !base.contains(&item.node.post) || &item.node.post == active)
        .collect();
    assert!(!blocked.is_empty());
    assert!(blocked
        .iter()
        .all(|item| !warp.admissible_action_ids.contains(&item.node.id)));
}
