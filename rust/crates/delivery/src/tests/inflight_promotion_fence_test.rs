use super::promotion_fixture::{response, PromotionFixture};
use super::support::transfer_identity;
use crate::manager::inflight::PromotionRejection;
use ghostr_engine::{ActionId, PostId};

#[tokio::test]
async fn direct_authorization_stages_headers_but_needs_selected_promotion() {
    let mut fixture = PromotionFixture::new(100).await;
    assert!(!fixture.active.authorizes_response(
        &fixture.attempt,
        &fixture.action,
        &response(),
        50
    ));
    assert!(fixture
        .active
        .preflight_promotion(&fixture.target, 50)
        .is_ok());
    assert!(!fixture.token.is_cancelled());
    fixture.cleanup().await;
}

#[tokio::test]
async fn observed_promotable_headers_publish_an_opportunity_without_admitting_body() {
    let mut fixture = PromotionFixture::new(100).await;
    let before = fixture.active.actions().remove(0);

    assert!(fixture
        .active
        .observe_headers(&fixture.attempt, &response(), 50));

    let after = fixture.active.actions().remove(0);
    assert_eq!(after.request(), before.request());
    assert_eq!(after.effective_bytes(), before.effective_bytes());
    assert_eq!(
        after.reserved_storage_bytes(),
        before.reserved_storage_bytes()
    );
    assert!(fixture
        .active
        .preflight_promotion(&fixture.target, 50)
        .is_ok());

    fixture.cleanup().await;
}

#[tokio::test]
async fn expired_or_reused_identity_is_refused_without_cancelling_primary() {
    let expired = PromotionFixture::new(100).await;
    assert!(matches!(
        expired.active.preflight_promotion(&expired.target, 101),
        Err(PromotionRejection::Expired)
    ));
    assert!(!expired.token.is_cancelled());
    expired.cleanup().await;

    let stale = PromotionFixture::new(100).await;
    let identity = transfer_identity(&PostId::new("post"), "https://stale.test/video");
    let target = stale.target.retarget(ActionId::new(1), identity);
    assert!(matches!(
        stale.active.preflight_promotion(&target, 50),
        Err(PromotionRejection::StaleIdentity)
    ));
    assert!(!stale.token.is_cancelled());
    stale.cleanup().await;
}
