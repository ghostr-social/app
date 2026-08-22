use crate::tests::warp_hedge_plan_fixture::{mirror_plan, HedgeCase};
use ghostr_engine::adaptive::PlannerCommand;

#[test]
fn hedge_waits_for_the_exact_primary_p95_completion_tail() {
    assert!(!generates_hedge(HedgeCase::BeforeTail));
    assert!(generates_hedge(HedgeCase::Eligible));
}

fn generates_hedge(case: HedgeCase) -> bool {
    mirror_plan(case)
        .warp
        .unwrap()
        .generated
        .actions
        .iter()
        .any(|action| matches!(action.command, PlannerCommand::Hedge { .. }))
}
