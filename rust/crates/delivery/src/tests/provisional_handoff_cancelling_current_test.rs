use crate::manager::inflight::ActiveAction;
use crate::tests::provisional_handoff_fixture::{cross_origin_handoff_state, CURRENT, NEXT};
use crate::tests::provisional_handoff_plan_fixture::{generated_cancels, plan};
use ghostr_engine::{ActionId, ByteRange, ChunkId, DataUsageLevel, PostId};
use std::collections::HashSet;

const CURRENT_SOURCE: &str = "https://current.example/current.mp4";

#[test]
fn cancelling_current_does_not_spend_its_replacement_slot() {
    let (state, futures) = cross_origin_handoff_state(DataUsageLevel::Conservative);
    let current = cancelling_current(&state);
    let active = [futures[0].clone(), futures[1].clone(), current];

    let work = plan(state, &active, 2);

    assert_eq!(generated_cancels(&work), HashSet::from([ActionId::new(1)]));
    assert!(work.retained.contains(&ActionId::new(2)));
    assert!(!work.retained.contains(&ActionId::new(1)));
    assert_eq!(
        work.retained_posts,
        HashSet::from([PostId::new(CURRENT), PostId::new(NEXT)])
    );
}

fn cancelling_current(state: &crate::manager::state::DeliveryState) -> ActiveAction {
    let post = PostId::new(CURRENT);
    let identity = state
        .catalog()
        .transfer_identity(&post, CURRENT_SOURCE)
        .expect("current representation");
    ActiveAction::range_with_action(
        ActionId::new(3),
        ChunkId {
            post,
            range: ByteRange::new(0, 65_536),
        },
        identity,
        4_000,
    )
    .cancelling_for_test()
}
