use super::promotion_fixture::{response, PromotionFixture};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::ByteRange;

#[tokio::test]
async fn selected_promotion_activates_only_before_response_headers() {
    let mut fixture = PromotionFixture::new(100).await;
    let preflight = fixture
        .active
        .preflight_promotion(&fixture.target, 50)
        .expect("exact active promotion");
    assert_eq!(preflight.additional_bytes(), 12);
    assert!(fixture.active.activate_promotion(&preflight, 50));
    assert!(fixture
        .active
        .authorizes_response(&fixture.attempt, &fixture.action, &response(), 50));

    let active = fixture.active.actions().remove(0);
    assert_eq!(
        active.request(),
        RetrievalRequest::FetchWhole {
            contract: WholeBodyContract::Capped { maximum_bytes: 16 },
            reason: WholeFetchReason::PromotedResponse,
        }
    );
    assert_eq!(active.effective_bytes(), ByteRange::new(0, 16));
    assert_eq!(active.reserved_storage_bytes(), 16);
    fixture.cleanup().await;
}

#[tokio::test]
async fn rejected_commit_can_restore_the_exact_range_authority() {
    let mut fixture = PromotionFixture::new(100).await;
    let preflight = fixture
        .active
        .preflight_promotion(&fixture.target, 50)
        .expect("valid test fixture");
    assert!(fixture.active.activate_promotion(&preflight, 50));
    assert!(fixture.active.rollback_promotion(&preflight));

    let active = fixture.active.actions().remove(0);
    assert_eq!(active.request().requested_bytes(), ByteRange::new(4, 8));
    assert_eq!(active.reserved_storage_bytes(), 4);
    assert!(fixture
        .active
        .preflight_promotion(&fixture.target, 50)
        .is_ok());
    fixture.cleanup().await;
}

#[tokio::test]
async fn rejected_commit_rolls_back_the_exact_store_delta_without_cancelling() {
    let mut fixture = PromotionFixture::new(100).await;
    let preflight = fixture
        .active
        .preflight_promotion(&fixture.target, 50)
        .expect("valid test fixture");
    let extension = fixture
        .store
        .extend_action(&fixture.action, 16)
        .await
        .expect("valid test fixture");
    assert_eq!(extension.additional_bytes(), 12);
    assert!(fixture.active.activate_promotion(&preflight, 50));
    assert!(fixture.active.rollback_promotion(&preflight));
    fixture.store.rollback_action(extension).await.expect("valid test fixture");

    let retry = fixture
        .store
        .extend_action(&fixture.action, 16)
        .await
        .expect("valid test fixture");
    assert_eq!(retry.additional_bytes(), 12);
    fixture.store.rollback_action(retry).await.expect("valid test fixture");
    assert!(!fixture.token.is_cancelled());
    fixture.cleanup().await;
}
