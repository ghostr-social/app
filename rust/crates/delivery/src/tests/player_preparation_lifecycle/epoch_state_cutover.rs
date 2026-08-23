use crate::delivery_events::{
    PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationObservation,
    PlayerPreparationReport, PlayerPreparationState,
};
use crate::tests::player_preparation_fixture::{focus, state};
use ghostr_engine::adaptive::PlayerPreparation;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::HashMap;

#[test]
fn newer_client_epoch_purges_old_manager_evidence_across_posts() {
    let mut delivery = state(&["p1", "p2"], 0);
    assert!(delivery.apply_player_preparation(report(
        &delivery,
        "p1",
        7,
        PlayerPreparationState::Initializing,
    )));

    assert!(delivery.apply_player_preparation(report(
        &delivery,
        "p2",
        8,
        PlayerPreparationState::Initializing,
    )));
    assert_eq!(
        delivery.player_preparation(&PostId::new("p1"), Some(ContentRevision::default())),
        PlayerPreparation::Unverified,
    );
    assert!(delivery.apply_player_preparation(report(
        &delivery,
        "p2",
        8,
        PlayerPreparationState::Released,
    )));
    delivery.prune_player_preparations(&HashMap::new());
    assert!(!delivery.apply_player_preparation(report(
        &delivery,
        "p1",
        7,
        PlayerPreparationState::Initialized,
    )));
    delivery.clear();
    delivery.apply_focus(focus(&["p1", "p2"], 0), 1);
    assert!(delivery.apply_player_preparation(report(
        &delivery,
        "p1",
        7,
        PlayerPreparationState::Initializing,
    )));
}

fn report(
    delivery: &crate::manager::state::DeliveryState,
    raw_post: &str,
    epoch: u64,
    state: PlayerPreparationState,
) -> PlayerPreparationReport {
    let sequence = u64::from(state != PlayerPreparationState::Initializing) + 1;
    let post = PostId::new(raw_post);
    let authority = PlayerPreparationAuthority::try_new(
        post.clone(),
        delivery.catalog().binding(&post).unwrap(),
        ContentRevision::default(),
        format!("asset-{raw_post}"),
    )
    .unwrap();
    let attempt = PlayerPreparationAttempt::try_new(1, epoch, 1).unwrap();
    let observation = PlayerPreparationObservation::try_new(state, None, sequence).unwrap();
    PlayerPreparationReport::try_new(authority, attempt, sequence, observation).unwrap()
}
