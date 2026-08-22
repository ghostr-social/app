use crate::tests::warp_hedge_plan_fixture::{mirror_plan, HedgeCase, ALTERNATE, PRIMARY_ACTION_ID};
use ghostr_engine::adaptive::PlannerCommand;

#[test]
fn delayed_verified_primary_selects_one_exact_alternate_hedge() {
    let work = mirror_plan(HedgeCase::Eligible);
    let decision = work.warp.expect("advanced decision");
    let selected = decision.selected.expect("tail hedge");
    let PlannerCommand::Hedge { primary, transfer } = selected.command else {
        panic!("expected hedge, got {:?}", selected.node.kind);
    };

    assert_eq!(primary, ghostr_engine::ActionId::new(PRIMARY_ACTION_ID));
    assert_eq!(transfer.source, ALTERNATE);
    assert_eq!(work.selected_transfers.len(), 1);
    assert_eq!(
        work.selected_transfers[0].identity.source().as_str(),
        ALTERNATE
    );
    assert_eq!(
        selected.node.resources.network_bytes,
        transfer.request.reserved_network_bytes()
    );
    assert_eq!(selected.node.resources.requests, 1);
}
