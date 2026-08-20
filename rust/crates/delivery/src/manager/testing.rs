use super::{DeliveryManagerConfig, DeliveryWorker};
use crate::delivery_events::CommandReceiver;
use crate::manager::timeline::{TimelineCoordinator, TimelineParser};
use crate::playback_demand::DemandReceiver;
use ghostr_engine::media_timeline::MediaTimeline;
use ghostr_engine::{EngineParams, PostId};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use std::sync::Arc;

impl DeliveryWorker {
    pub(crate) async fn create_with_timeline_parser<C>(
        config: DeliveryManagerConfig<C>,
        commands: CommandReceiver,
        demand: DemandReceiver,
        parser: Arc<dyn TimelineParser>,
    ) -> Self
    where
        C: MediaHttpRequests + 'static,
    {
        let store = config.store.clone();
        let mut worker = Self::create(config, commands, demand).await;
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
