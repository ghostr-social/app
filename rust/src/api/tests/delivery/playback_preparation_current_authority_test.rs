use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlaybackPreparationReadiness;
use crate::api::playback_preparation_stream::{projection, PreparationContext};
use ghostr_delivery::delivery_events::PlayerPreparationClaim;
use ghostr_engine::PostId;

#[tokio::test]
async fn renewed_current_asset_does_not_inherit_an_old_first_frame() {
    let mut fixture = AuthorityFixture::seeded().await;
    fixture
        .commands
        .publish_focused_plan_with_player_preparations(
            7,
            Some(PostId::new("clip")),
            Default::default(),
            (vec![claim(&fixture)], Vec::new()),
        );
    let context = context(&fixture);
    assert_eq!(
        readiness(&context).await,
        FfiPlaybackPreparationReadiness::Ready
    );

    let renewed = fixture.renew_content_revision().await;
    assert_ne!(renewed, fixture.asset);
    assert_eq!(
        readiness(&context).await,
        FfiPlaybackPreparationReadiness::Preparing,
    );
}

fn claim(fixture: &AuthorityFixture) -> PlayerPreparationClaim {
    PlayerPreparationClaim::try_new(
        PostId::new("clip"),
        fixture.representation.clone(),
        &fixture.asset,
    )
    .expect("player preparation claim")
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

async fn readiness(context: &PreparationContext) -> FfiPlaybackPreparationReadiness {
    projection::project(context)
        .await
        .expect("preparation plan")
        .current
        .expect("current asset")
        .readiness
}
