use super::{DeliveryManagerConfig, DeliveryWorker};
use crate::delivery_events::CommandReceiver;
use crate::manager::timeline::{TimelineCoordinator, TimelineParser};
use crate::playback_demand::DemandReceiver;
use ghostr_engine::media_timeline::MediaTimeline;
use ghostr_engine::{EngineParams, PostId};
use std::sync::Arc;

mod response;

impl DeliveryWorker {
    pub(crate) async fn create_with_timeline_parser(
        config: DeliveryManagerConfig,
        commands: CommandReceiver,
        demand: DemandReceiver,
        parser: Arc<dyn TimelineParser>,
    ) -> Self {
        let store = config.store.clone();
        let resources = crate::manager::resource_control::ResourceControl::bootstrap(
            &config,
            tokio::time::Instant::now(),
        );
        assert!(config
            .requests
            .install_resource_observer(Arc::new(resources.clone())));
        let mut worker = Self::create(config, commands, demand, resources).await;
        worker.timelines = TimelineCoordinator::with_parser(store, parser, 2);
        worker
    }

    pub(crate) fn timeline_for_test(&self, post: &PostId) -> Option<MediaTimeline> {
        self.state.catalog().lookup(post)?.timeline().cloned()
    }

    pub(crate) fn current_post_for_test(&self) -> Option<&PostId> {
        self.state.focus().current()
    }

    pub(crate) fn params_for_test(&self) -> EngineParams {
        *self.state.params()
    }

    pub(crate) fn timeline_result_ready_for_test(&mut self) -> bool {
        self.timelines.prepare_wake()
    }
}
