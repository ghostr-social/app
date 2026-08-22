use super::promotion_fixture::{response, PromotionFixture};
use super::support::transfer_identity;
use crate::manager::inflight::PromotionRejection;
use ghostr_engine::{ActionId, PostId};

#[tokio::test]
async fn latent_grant_cannot_authorize_headers_or_be_reused_after_open() {
    let mut fixture = PromotionFixture::new(100).await;
    assert!(!fixture.active.authorizes_response(
        &fixture.attempt,
        &fixture.action,
        &response(),
        50
    ));
    assert!(matches!(
        fixture.active.preflight_promotion(&fixture.target, 50),
        Err(PromotionRejection::ResponseOpened)
    ));
    assert!(!fixture.token.is_cancelled());
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
