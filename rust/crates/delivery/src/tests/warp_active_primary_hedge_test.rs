use crate::tests::warp_hedge_plan_fixture::{mirror_plan, HedgeCase};
use ghostr_engine::adaptive::PlannerCommand;

#[test]
fn a_blocked_primary_does_not_authorize_unauthenticated_alternate_ranges() {
    let decision = mirror_plan(HedgeCase::PrimaryUnavailable)
        .warp
        .expect("advanced decision");
    let hedge = decision.generated.actions.iter().find_map(|action| {
        let PlannerCommand::Hedge { transfer, .. } = &action.command else {
            return None;
        };
        Some(transfer)
    });

    assert!(hedge.is_none());
}
