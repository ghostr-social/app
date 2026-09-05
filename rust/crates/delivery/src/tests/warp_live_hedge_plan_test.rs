use crate::tests::warp_hedge_plan_fixture::{mirror_plan, HedgeCase};
use ghostr_engine::adaptive::PlannerCommand;

#[test]
fn past_complete_verification_does_not_authorize_a_new_sparse_race() {
    assert_no_hedge(mirror_plan(HedgeCase::Eligible));
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
