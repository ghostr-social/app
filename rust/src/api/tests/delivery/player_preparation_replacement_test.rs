use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlayerPreparationState;
use crate::api::player_preparation_control::report_player_preparation;
use ghostr_delivery::delivery_events::PlayerPreparationState;

#[tokio::test]
async fn renewed_attempt_releases_the_old_authority_before_replacing_it() {
    let mut fixture = AuthorityFixture::seeded().await;
    let old = fixture.input();
    report_player_preparation(&fixture.context, old.clone())
        .await
        .unwrap();
    fixture.commands.try_player_preparation().unwrap();
    let renewed = fixture.renew_content_revision().await;

    let mut replacement = old.clone();
    replacement.asset_id = renewed;
    replacement.attempt_generation = 4;
    report_player_preparation(&fixture.context, replacement)
        .await
        .unwrap();
    let released = fixture.commands.try_player_preparation().unwrap();
    assert_eq!(released.state(), PlayerPreparationState::Released);
    let initializing = fixture.commands.try_player_preparation().unwrap();
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
