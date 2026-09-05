use super::capability_renewal_fixture::{context, evict_capability};
use super::playback_preparation_sparse_fixture::complete_startup;
use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlaybackPreparationReadiness;
use crate::api::playback_preparation_stream::projection;
use crate::api::tests::support::sized_meta;
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
    let snapshot = fixture
        .context
        .store
        .media_snapshot("clip")
        .await
        .expect("fixture");
    let startup = complete_startup(&sized_meta(16, 2_000), 16);
    let certificate = StartupCertificate::issue(startup.clone(), &snapshot).expect("fixture");
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
    .expect("fixture")
}
