use crate::manager::inflight::ActiveAction;
use crate::tests::provisional_handoff_fixture::{detached_next, DetachedFuture};
use crate::tests::provisional_handoff_plan_fixture::{generated_cancels, plan};
use ghostr_engine::{ActionId, ByteRange, ChunkId};
use std::collections::HashSet;

#[test]
fn duplicate_future_legs_consume_distinct_request_slots() {
    let DetachedFuture { state, active } = detached_next(4_000, None);
    let alternate = ActiveAction::range_with_action(
        ActionId::new(2),
        ChunkId {
            post: active.post().clone(),
            range: active.effective_bytes(),
        },
        active.identity().clone(),
        active.committed_until_ms(),
    )
    .hedged_for_test()
    .effective_bytes_for_test(ByteRange::new(32_768, 65_536));
    let active = [active.hedged_for_test(), alternate];

    let work = plan(state, &active, 3);

    assert!(work.retained.contains(&ActionId::new(2)));
    assert!(!work.retained.contains(&ActionId::new(1)));
    assert_eq!(generated_cancels(&work), HashSet::from([ActionId::new(1)]));
}
