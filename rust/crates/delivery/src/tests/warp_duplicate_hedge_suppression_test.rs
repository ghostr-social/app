use crate::tests::warp_hedge_plan_fixture::{mirror_plan, HedgeCase};
use ghostr_engine::adaptive::PlannerCommand;

#[test]
fn an_existing_linked_hedge_suppresses_another_alternate() {
    let work = mirror_plan(HedgeCase::Linked);
    assert!(work
        .warp
        .unwrap()
        .generated
        .actions
        .iter()
        .all(|action| !matches!(action.command, PlannerCommand::Hedge { .. })));
}
