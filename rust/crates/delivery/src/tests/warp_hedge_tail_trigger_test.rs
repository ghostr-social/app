use crate::tests::warp_hedge_plan_fixture::{mirror_plan, HedgeCase};
use ghostr_engine::adaptive::PlannerCommand;

#[test]
fn passing_the_primary_tail_does_not_enable_an_unverified_range_race() {
    assert!(!generates_hedge(HedgeCase::BeforeTail));
    assert!(!generates_hedge(HedgeCase::Eligible));
}

fn generates_hedge(case: HedgeCase) -> bool {
    mirror_plan(case)
        .warp
        .expect("fixture")
        .generated
        .actions
        .iter()
        .any(|action| matches!(action.command, PlannerCommand::Hedge { .. }))
}
