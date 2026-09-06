use super::playback_preparation_current_lifecycle_fixture::context;
use super::playback_preparation_history_fixture::focused_history;
use crate::api::delivery_types::FfiPlaybackPreparationReadiness as Readiness;
use crate::api::playback_preparation_stream::projection;

#[tokio::test]
async fn moving_forward_keeps_the_previous_cached_video_available_for_preparation() {
    let fixture = focused_history().await;
    let evidence = fixture
        .manager
        .context
        .delivery
        .latest_plan()
        .expect("plan");
    assert!(evidence.plan.ready_reserve.candidates.is_empty());
    let projected = projection::project(&context(&fixture.manager))
        .await
        .expect("plan");
    assert!(
        projected.next.is_none(),
        "history is not a forward rescue candidate"
    );
    let previous = projected
        .upcoming
        .iter()
        .find(|asset| asset.delivery_id == "clip");
    let result = previous.map(|asset| (asset.asset_id.clone(), asset.readiness));
    let original_asset = fixture.manager.input.asset_id.clone();
    fixture.shutdown().await;
    assert_eq!(
        result,
        Some((original_asset, Readiness::Ready)),
        "a backward preview needs the same authorized asset after focus moves"
    );
}
