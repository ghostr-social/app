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

#[test]
fn advertised_hash_alone_cannot_authorize_sparse_mirror_hedging() {
    let work = mirror_plan(HedgeCase::AdvertisedOnly);

    assert_no_hedge(work);
}

#[test]
fn one_verified_source_cannot_authorize_ranges_from_an_unverified_alternate() {
    let work = mirror_plan(HedgeCase::PrimaryVerifiedOnly);

    assert_no_hedge(work);
}

#[test]
fn alternate_validator_rotation_revokes_sparse_mirror_authority() {
    let work = mirror_plan(HedgeCase::AlternateRotated);

    assert_no_hedge(work);
}

#[test]
fn terminal_primary_cannot_schedule_or_launch_an_alternate_hedge() {
    let work = mirror_plan(HedgeCase::Terminal);

    assert!(work.hedge_tails.is_empty());
    assert_no_hedge(work);
}

fn assert_no_hedge(work: crate::manager::plan::PlannedWork) {
    assert!(work
        .warp
        .expect("advanced decision")
        .generated
        .actions
        .iter()
        .all(|action| !matches!(action.command, PlannerCommand::Hedge { .. })));
}
