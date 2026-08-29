use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlaybackPreparationReadiness;
use crate::api::playback_preparation_stream::{projection, PreparationContext};
use crate::api::tests::support::{bind_store, sized_meta};
use core::time::Duration;
use ghostr_delivery::delivery_events::PlayerPreparationClaim;
use ghostr_engine::PostId;
use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};

#[tokio::test]
async fn renewed_current_capability_does_not_inherit_an_old_first_frame() {
    let limits = ProgressiveCapabilityLimits::new(1, Duration::from_secs(60)).unwrap();
    let mut fixture = AuthorityFixture::seeded_with(ProgressiveCapabilities::new(limits)).await;
    publish_verified_current(&mut fixture);
    let context = context(&fixture);
    let initial = projection::project(&context)
        .await
        .unwrap()
        .current
        .unwrap();
    assert_eq!(initial.readiness, FfiPlaybackPreparationReadiness::Ready);

    evict_capability(&fixture).await;
    let renewed = projection::project(&context)
        .await
        .unwrap()
        .current
        .unwrap();
    assert_ne!(renewed.asset_id, initial.asset_id);
    assert_eq!(
        renewed.readiness,
        FfiPlaybackPreparationReadiness::Preparing,
    );
}

fn publish_verified_current(fixture: &mut AuthorityFixture) {
    let claim = PlayerPreparationClaim::try_new(
        PostId::new("clip"),
        fixture.representation.clone(),
        &fixture.asset,
    )
    .unwrap();
    fixture
        .commands
        .publish_focused_plan_with_player_preparations(
            7,
            Some(PostId::new("clip")),
            Default::default(),
            (vec![claim], Vec::new()),
        );
}

async fn evict_capability(fixture: &AuthorityFixture) {
    let meta = sized_meta(16, 2_000);
    bind_store(&fixture.context.store, "other", &meta).await;
    fixture
        .context
        .store
        .set_total_len("other", 16)
        .await
        .unwrap();
    fixture
        .context
        .store
        .write_range("other", 0, &[3; 16])
        .await
        .unwrap();
    let snapshot = fixture.context.store.media_snapshot("other").await.unwrap();
    fixture.context.capabilities.issue(&snapshot).await.unwrap();
}

fn context(fixture: &AuthorityFixture) -> PreparationContext {
    PreparationContext {
        endpoint: "127.0.0.1:8080".to_owned(),
        store: std::sync::Arc::clone(&fixture.context.store),
        capabilities: fixture.context.capabilities.clone(),
        delivery: fixture.context.delivery.clone(),
        tracked: fixture.context.tracked.clone(),
        cache: fixture.context.cache.clone(),
    }
}
