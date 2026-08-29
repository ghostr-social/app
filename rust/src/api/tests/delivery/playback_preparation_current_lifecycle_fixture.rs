use super::player_preparation_manager_fixture::ProductionManagerFixture;
use crate::api::delivery_types::{
    FfiPlaybackPreparationAsset, FfiPlaybackPreparationPlan, FfiPlaybackPreparationReadiness,
    FfiPlayerPreparationDisposition, FfiPlayerPreparationState,
};
use crate::api::playback_preparation_stream::{
    watch_preparation, PreparationContext, PreparationOut,
};
use crate::api::player_preparation_control::confirm_player_preparation;
use core::time::Duration;
use tokio::sync::mpsc;

struct ChannelOut(mpsc::UnboundedSender<FfiPlaybackPreparationPlan>);

impl PreparationOut for ChannelOut {
    fn send(&self, plan: FfiPlaybackPreparationPlan) -> bool {
        self.0.send(plan).is_ok()
    }
}

pub(super) struct CurrentLifecycleFixture {
    pub(super) manager: ProductionManagerFixture,
    plans: mpsc::UnboundedReceiver<FfiPlaybackPreparationPlan>,
    watcher: tokio::task::JoinHandle<()>,
}

impl CurrentLifecycleFixture {
    pub(super) async fn start() -> Self {
        let manager = ProductionManagerFixture::seeded().await;
        let (sender, plans) = mpsc::unbounded_channel();
        let watcher = tokio::spawn(watch_preparation(ChannelOut(sender), context(&manager)));
        Self {
            manager,
            plans,
            watcher,
        }
    }

    pub(super) async fn render_first_frame(&mut self) {
        self.report(1, FfiPlayerPreparationState::Initializing)
            .await;
        self.report(2, FfiPlayerPreparationState::FirstFrameRendered)
            .await;
        self.wait_for_current("clip", FfiPlaybackPreparationReadiness::Ready)
            .await;
    }

    pub(super) async fn report(&self, sequence: u64, state: FfiPlayerPreparationState) {
        let mut input = self.manager.input.clone();
        input.sequence = sequence;
        input.state = state;
        input.failure_kind =
            (state == FfiPlayerPreparationState::Failed).then(|| "decoderFailure".to_owned());
        let result = confirm_player_preparation(&self.manager.context, input).await;
        assert_eq!(result, FfiPlayerPreparationDisposition::Applied);
    }

    pub(super) async fn wait_for_current(
        &mut self,
        delivery_id: &str,
        readiness: FfiPlaybackPreparationReadiness,
    ) -> FfiPlaybackPreparationAsset {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let plan = self.plans.recv().await?;
                let Some(asset) = plan.current else {
                    continue;
                };
                if asset.delivery_id == delivery_id && asset.readiness == readiness {
                    return Some(asset);
                }
            }
        })
        .await
        .expect("preparation deadline")
        .expect("matching current preparation")
    }

    pub(super) async fn shutdown(self) {
        self.watcher.abort();
        self.manager.shutdown().await;
    }
}

pub(super) fn context(fixture: &ProductionManagerFixture) -> PreparationContext {
    PreparationContext {
        endpoint: "127.0.0.1:8080".to_owned(),
        store: std::sync::Arc::clone(&fixture.context.store),
        capabilities: fixture.context.capabilities.clone(),
        delivery: fixture.context.delivery.clone(),
        tracked: fixture.context.tracked.clone(),
        cache: fixture.context.cache.clone(),
    }
}
