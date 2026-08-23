use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlayerPreparationState;
use crate::api::player_preparation_control::report_player_preparation;
use ghostr_delivery::delivery_events::PlayerPreparationState;

#[tokio::test]
async fn admitted_attempt_finishes_under_its_original_revision_after_renewal() {
    let mut fixture = AuthorityFixture::seeded().await;
    let original = fixture
        .context
        .store
        .media_snapshot("clip")
        .await
        .unwrap()
        .revision();
    let mut report = fixture.input();
    report_player_preparation(&fixture.context, report.clone())
        .await
        .unwrap();
    fixture.commands.try_player_preparation().unwrap();
    let renewed = fixture.renew_content_revision().await;
    assert_ne!(renewed, fixture.asset);

    report.sequence = 2;
    report.state = FfiPlayerPreparationState::FirstFrameRendered;
    report_player_preparation(&fixture.context, report.clone())
        .await
        .unwrap();
    let rendered = fixture.commands.try_player_preparation().unwrap();
    assert_eq!(rendered.state(), PlayerPreparationState::FirstFrameRendered);
    assert_eq!(rendered.revision(), original);

    report.sequence = 3;
    report.state = FfiPlayerPreparationState::Released;
    report_player_preparation(&fixture.context, report.clone())
        .await
        .unwrap();
    let released = fixture.commands.try_player_preparation().unwrap();
    assert_eq!(released.state(), PlayerPreparationState::Released);
    assert_eq!(released.revision(), original);

    report.sequence = 4;
    assert!(report_player_preparation(&fixture.context, report)
        .await
        .is_err());
    assert!(fixture.commands.try_player_preparation().is_none());
}

#[tokio::test]
async fn renewed_asset_cannot_forge_a_follow_up_for_an_admitted_attempt() {
    let mut fixture = AuthorityFixture::seeded().await;
    let begin = fixture.input();
    report_player_preparation(&fixture.context, begin.clone())
        .await
        .unwrap();
    fixture.commands.try_player_preparation().unwrap();
    let renewed = fixture.renew_content_revision().await;
    let mut forged = begin;
    forged.asset_id = renewed;
    forged.sequence = 2;
    forged.state = FfiPlayerPreparationState::FirstFrameRendered;
    assert!(report_player_preparation(&fixture.context, forged)
        .await
        .is_err());
}

#[tokio::test]
async fn unseen_attempt_cannot_claim_first_frame_evidence() {
    let mut fixture = AuthorityFixture::seeded().await;
    let mut unseen = fixture.input();
    unseen.sequence = 2;
    unseen.state = FfiPlayerPreparationState::FirstFrameRendered;

    assert!(report_player_preparation(&fixture.context, unseen)
        .await
        .is_err());
    assert!(fixture.commands.try_player_preparation().is_none());
}

#[tokio::test]
async fn admitted_follow_up_survives_current_cache_removal() {
    let mut fixture = AuthorityFixture::seeded().await;
    let mut report = fixture.input();
    report_player_preparation(&fixture.context, report.clone())
        .await
        .unwrap();
    fixture.commands.try_player_preparation().unwrap();
    fixture.context.cache.replace(Vec::new());
    report.sequence = 2;
    report.state = FfiPlayerPreparationState::Released;

    report_player_preparation(&fixture.context, report)
        .await
        .expect("admitted authority owns its terminal lifecycle");

    let released = fixture.commands.try_player_preparation().unwrap();
    assert_eq!(released.state(), PlayerPreparationState::Released);
}
