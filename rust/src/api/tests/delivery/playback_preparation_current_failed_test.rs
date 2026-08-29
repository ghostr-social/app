use super::playback_preparation_current_lifecycle_fixture::CurrentLifecycleFixture;
use crate::api::delivery_types::{FfiPlaybackPreparationReadiness, FfiPlayerPreparationState};

#[tokio::test]
async fn failed_current_first_frame_returns_to_preparing() {
    let mut fixture = CurrentLifecycleFixture::start().await;
    fixture.render_first_frame().await;

    fixture.report(3, FfiPlayerPreparationState::Failed).await;
    fixture
        .wait_for_current("clip", FfiPlaybackPreparationReadiness::Preparing)
        .await;

    fixture.shutdown().await;
}
