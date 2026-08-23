use crate::api::delivery_types::FfiPlayerPreparationState;
use crate::api::player_preparation_control::report_player_preparation;
use crate::api::tests::delivery::player_preparation_authority_fixture::AuthorityFixture;
use ghostr_delivery::delivery_events::PlayerPreparationState;

#[tokio::test]
async fn feedback_requires_the_exact_live_asset_authority() {
    let mut fixture = AuthorityFixture::seeded().await;
    assert_mapping(&mut fixture).await;
    assert_invalid_authorities(&fixture).await;
    let eviction = 8..16;
    fixture
        .context
        .store
        .evict_ranges("clip", std::slice::from_ref(&eviction))
        .await
        .unwrap();
    assert!(report_player_preparation(&fixture.context, fixture.input())
        .await
        .is_err());
}

async fn assert_mapping(fixture: &mut AuthorityFixture) {
    report_player_preparation(&fixture.context, fixture.input())
        .await
        .unwrap();
    let report = fixture.commands.try_player_preparation().unwrap();
    assert_eq!(report.post().as_str(), "clip");
    assert_eq!(
        report.binding().representation().fingerprint(),
        fixture.representation
    );
    assert_eq!(report.player_capability_generation(), 1);
    assert_eq!(report.client_epoch(), 2);
    assert_eq!(report.attempt_generation(), 3);
    assert_eq!(report.sequence(), 1);
    assert_eq!(report.observed_monotonic_us(), 5);
    assert_eq!(report.state(), PlayerPreparationState::Initializing);
    let mut rendered = fixture.input();
    rendered.sequence = 2;
    rendered.state = FfiPlayerPreparationState::FirstFrameRendered;
    report_player_preparation(&fixture.context, rendered)
        .await
        .unwrap();
    let rendered = fixture.commands.try_player_preparation().unwrap();
    assert_eq!(rendered.state(), PlayerPreparationState::FirstFrameRendered);
}

async fn assert_invalid_authorities(fixture: &AuthorityFixture) {
    let mut wrong_representation = fixture.input();
    wrong_representation.representation_id = "wrong".to_owned();
    assert!(
        report_player_preparation(&fixture.context, wrong_representation)
            .await
            .is_err()
    );
    let mut wrong_asset = fixture.input();
    wrong_asset.asset_id = "wrong".to_owned();
    assert!(report_player_preparation(&fixture.context, wrong_asset)
        .await
        .is_err());
    let mut invalid_failure = fixture.input();
    invalid_failure.state = FfiPlayerPreparationState::Failed;
    assert!(report_player_preparation(&fixture.context, invalid_failure)
        .await
        .is_err());
    let mut wrong_post = fixture.input();
    wrong_post.post_id = "other".to_owned();
    assert!(report_player_preparation(&fixture.context, wrong_post)
        .await
        .is_err());
}
