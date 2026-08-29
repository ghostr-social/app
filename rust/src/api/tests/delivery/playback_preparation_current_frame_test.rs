use super::player_preparation_manager_fixture::ProductionManagerFixture;
use crate::api::delivery_types::{
    FfiPlaybackPreparationAsset, FfiPlaybackPreparationPlan, FfiPlaybackPreparationReadiness,
    FfiPlayerPreparationState,
};
use crate::api::playback_preparation_stream::{
    watch_preparation, PreparationContext, PreparationOut,
};
use crate::api::player_preparation_control::confirm_player_preparation;
use core::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

struct ChannelOut(mpsc::UnboundedSender<FfiPlaybackPreparationPlan>);

impl PreparationOut for ChannelOut {
    fn send(&self, plan: FfiPlaybackPreparationPlan) -> bool {
        self.0.send(plan).is_ok()
    }
}

#[tokio::test]
async fn current_first_frame_projects_player_verified_readiness() {
    let fixture = ProductionManagerFixture::seeded().await;
    let (sender, mut plans) = mpsc::unbounded_channel();
    let watcher = tokio::spawn(watch_preparation(ChannelOut(sender), context(&fixture)));
    let initial = next_plan(&mut plans).await;
    assert_eq!(
        initial.current.expect("current asset").readiness,
        FfiPlaybackPreparationReadiness::Preparing,
    );
    report(&fixture, 1, FfiPlayerPreparationState::Initializing).await;
    report(&fixture, 2, FfiPlayerPreparationState::FirstFrameRendered).await;
    let ready = wait_for_ready(&mut plans).await.expect("ready asset");
    watcher.abort();
    drop(plans);
    assert_eq!(ready.asset_id, fixture.input.asset_id);
    assert_eq!(ready.representation_id, fixture.input.representation_id);
    fixture.shutdown().await;
    assert_eq!(ready.readiness, FfiPlaybackPreparationReadiness::Ready);
}

fn context(fixture: &ProductionManagerFixture) -> PreparationContext {
    PreparationContext {
        endpoint: "127.0.0.1:8080".to_owned(),
        store: std::sync::Arc::clone(&fixture.context.store),
        capabilities: fixture.context.capabilities.clone(),
        delivery: fixture.context.delivery.clone(),
        tracked: fixture.context.tracked.clone(),
        cache: fixture.context.cache.clone(),
    }
}

async fn report(
    fixture: &ProductionManagerFixture,
    sequence: u64,
    state: FfiPlayerPreparationState,
) {
    let mut input = fixture.input.clone();
    input.sequence = sequence;
    input.state = state;
    let result = confirm_player_preparation(&fixture.context, input).await;
    assert!(matches!(
        result,
        crate::api::delivery_types::FfiPlayerPreparationDisposition::Applied
    ));
}

async fn next_plan(
    plans: &mut mpsc::UnboundedReceiver<FfiPlaybackPreparationPlan>,
) -> FfiPlaybackPreparationPlan {
    timeout(Duration::from_secs(2), plans.recv())
        .await
        .expect("preparation deadline")
        .expect("preparation plan")
}

async fn wait_for_ready(
    plans: &mut mpsc::UnboundedReceiver<FfiPlaybackPreparationPlan>,
) -> Option<FfiPlaybackPreparationAsset> {
    timeout(Duration::from_secs(2), async {
        loop {
            let current = plans.recv().await?.current?;
            if current.readiness == FfiPlaybackPreparationReadiness::Ready {
                return Some(current);
            }
        }
    })
    .await
    .ok()
    .flatten()
}
