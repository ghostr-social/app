use super::capability_renewal_fixture::{context, evict_capability};
use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::delivery_types::FfiPlaybackPreparationReadiness;
use crate::api::playback_preparation_stream::projection;
use core::time::Duration;
use ghostr_delivery::delivery_events::PlayerPreparationClaim;
use ghostr_engine::PostId;
use ghostr_gateway::progressive::capabilities::{
    ProgressiveCapabilities, ProgressiveCapabilityLimits,
};

#[tokio::test]
async fn renewed_current_capability_does_not_inherit_an_old_first_frame() {
    let limits = ProgressiveCapabilityLimits::new(1, Duration::from_secs(60)).expect("fixture");
    let mut fixture = AuthorityFixture::seeded_with(ProgressiveCapabilities::new(limits)).await;
    publish_verified_current(&mut fixture);
    let context = context(&fixture);
    let initial = projection::project(&context)
        .await
        .expect("fixture")
        .current
        .expect("fixture");
    assert_eq!(initial.readiness, FfiPlaybackPreparationReadiness::Ready);

    evict_capability(&fixture).await;
    let renewed = projection::project(&context)
        .await
        .expect("fixture")
        .current
        .expect("fixture");
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
    .expect("fixture");
    fixture
        .commands
        .publish_focused_plan_with_player_preparations(
            7,
            Some(PostId::new("clip")),
            Default::default(),
            (vec![claim], Vec::new()),
        );
}
