use super::playback_preparation_history_fixture::{focused_history, previous_asset, wait_for_plan};
use crate::api::delivery_types::{
    FfiPlaybackPreparationReadiness as Readiness, FfiPlayerPreparationState,
};

#[tokio::test]
async fn a_released_previous_player_keeps_only_structural_preparation_credit() {
    let fixture = focused_history().await;
    assert_eq!(
        previous_asset(&fixture).await.expect("previous").readiness,
        Readiness::Ready
    );
    fixture.report(3, FfiPlayerPreparationState::Released).await;
    wait_for_plan(&fixture.manager.context.delivery, |plan| {
        plan.player_preparations
            .iter()
            .all(|claim| claim.post().as_str() != "clip")
    })
    .await;
    let previous = previous_asset(&fixture).await.expect("cached startup");
    assert_eq!(previous.asset_id, fixture.manager.input.asset_id);
    fixture.shutdown().await;
    assert_eq!(previous.readiness, Readiness::StructuralStartable);
}
