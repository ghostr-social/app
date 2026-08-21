use crate::tests::warp_hedge_plan_fixture::{mirror_plan, HedgeCase, ALTERNATE};
use ghostr_engine::adaptive::PlannerCommand;

#[test]
fn blocked_but_active_primary_can_still_launch_an_available_alternate() {
    let decision = mirror_plan(HedgeCase::PrimaryUnavailable)
        .warp
        .expect("advanced decision");
    let hedge = decision.generated.actions.iter().find_map(|action| {
        let PlannerCommand::Hedge { transfer, .. } = &action.command else {
            return None;
        };
        Some(transfer)
    });

    assert_eq!(hedge.expect("tail hedge").source, ALTERNATE);
}
