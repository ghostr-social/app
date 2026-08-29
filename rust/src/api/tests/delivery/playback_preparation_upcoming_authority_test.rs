use super::playback_preparation_sparse_fixture::complete_startup;
use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlaybackPreparationReadiness;
use crate::api::playback_preparation_stream::{projection, PreparationContext};
use crate::api::tests::support::{bind_store, sized_meta};
use core::time::Duration;
use ghostr_delivery::delivery_events::PlayerPreparationClaim;
use ghostr_delivery::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{AllocationPlan, NextReserveEvidence};
use ghostr_engine::PostId;
use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};

#[tokio::test]
async fn renewed_upcoming_capability_downgrades_old_player_evidence() {
    let limits =
        ProgressiveCapabilityLimits::new(1, Duration::from_secs(60)).expect("capability limits");
    let mut fixture = AuthorityFixture::seeded_with(ProgressiveCapabilities::new(limits)).await;
    publish_ready_plan(&mut fixture).await;
    let context = context(&fixture);
    let initial = projection::project(&context).await.expect("initial plan");
    let initial = initial.next.expect("initial asset");
    assert_eq!(initial.readiness, FfiPlaybackPreparationReadiness::Ready);

    evict_capability(&fixture).await;
    let renewed = projection::project(&context).await.expect("renewed plan");
    let renewed = renewed.next.expect("renewed asset");
    assert_ne!(renewed.asset_id, initial.asset_id);
    assert_eq!(
        renewed.readiness,
        FfiPlaybackPreparationReadiness::StructuralStartable,
    );
}

async fn publish_ready_plan(fixture: &mut AuthorityFixture) {
    let snapshot = fixture.context.store.media_snapshot("clip").await.unwrap();
    let startup = complete_startup(&sized_meta(16, 2_000), 16);
    let certificate = StartupCertificate::issue(startup.clone(), &snapshot).unwrap();
    let plan = AllocationPlan {
        next_reserve: NextReserveEvidence::Ready {
            post: PostId::new("clip"),
            startup,
        },
        ..Default::default()
    };
    let claim = claim(fixture);
    fixture
        .commands
        .publish_focused_plan_with_player_preparations(
            7,
            None,
            plan,
            (vec![claim], vec![certificate]),
        );
}

fn claim(fixture: &AuthorityFixture) -> PlayerPreparationClaim {
    PlayerPreparationClaim::try_new(
        PostId::new("clip"),
        fixture.representation.clone(),
        &fixture.asset,
    )
    .unwrap()
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
