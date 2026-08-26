
use crate::delivery_events::{PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState};
use crate::tests::player_preparation_fixture::{focus, state};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;

#[test]
fn newer_epoch_retires_old_evidence_even_when_its_first_post_left_scope() {
    let mut delivery = state(&["p1", "p2"], 0);
    let old = report(&delivery, spec("p1", 7, 1, PlayerPreparationState::Initializing));
    let cutover = report(&delivery, spec("p2", 8, 1, PlayerPreparationState::Initializing));
    assert!(delivery.apply_player_preparation(old));
    delivery.apply_focus(focus(&["p1"], 0), 2);

    assert!(!delivery.apply_player_preparation(cutover));
    assert!(!delivery.apply_player_preparation(report(
        &delivery,
        spec("p1", 7, 2, PlayerPreparationState::Initialized),
    )));
}

#[derive(Clone, Copy)]
struct ReportSpec<'a> {
    post: &'a str,
    epoch: u64,
    sequence: u64,
    state: PlayerPreparationState,
}

fn spec(
    post: &str,
    epoch: u64,
    sequence: u64,
    state: PlayerPreparationState,
) -> ReportSpec<'_> {
    ReportSpec {
        post,
        epoch,
        sequence,
        state,
    }
}

fn report(
    delivery: &crate::manager::state::DeliveryState,
    spec: ReportSpec<'_>,
) -> PlayerPreparationReport {
    let post = PostId::new(spec.post);
    let authority = PlayerPreparationAuthority::try_new(
        post.clone(),
        delivery.catalog().binding(&post).expect("valid test fixture"),
        ContentRevision::default(),
        format!("asset-{}", spec.post),
    )
    .expect("valid test fixture");
    let attempt = PlayerPreparationAttempt::try_new(1, spec.epoch, 1).expect("valid test fixture");
    let observation =
        PlayerPreparationObservation::try_new(spec.state, None, spec.sequence).expect("valid test fixture");
    PlayerPreparationReport::try_new(authority, attempt, spec.sequence, observation).expect("valid test fixture")
}
