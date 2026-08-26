use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlayerPreparationState;
use crate::api::player_preparation_control::axiom_test_support::report_player_preparation;
use ghostr_delivery::delivery_events::PlayerPreparationState;

#[tokio::test]
async fn renewed_attempt_releases_the_old_authority_before_replacing_it() {
    let fixture = AuthorityFixture::seeded().await;
    let old = fixture.input();
    report_player_preparation(&fixture.context, old.clone())
        .await
        .expect("test fixture precondition must hold");
    fixture
        .commands
        .try_player_preparation()
        .expect("test fixture precondition must hold");
    let renewed = fixture.renew_content_revision().await;

    let mut replacement = old.clone();
    replacement.asset_id = renewed;
    replacement.attempt_generation = 4;
    report_player_preparation(&fixture.context, replacement)
        .await
        .expect("test fixture precondition must hold");
    let released = fixture
        .commands
        .try_player_preparation()
        .expect("test fixture precondition must hold");
    assert_eq!(released.state(), PlayerPreparationState::Released);
    let initializing = fixture
        .commands
        .try_player_preparation()
        .expect("test fixture precondition must hold");
    assert_eq!(initializing.state(), PlayerPreparationState::Initializing);
    assert_eq!(initializing.attempt_generation(), 4);

    let mut late = old;
    late.sequence = 2;
    late.state = FfiPlayerPreparationState::FirstFrameRendered;
    assert!(report_player_preparation(&fixture.context, late)
        .await
        .is_err());
    assert!(fixture.commands.try_player_preparation().is_none());
}
