use super::fallback_environment;
use crate::delivery_events::{
    command_channel, DeliveryHandle, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationIngress, PlayerPreparationObservation, PlayerPreparationReport,
    PlayerPreparationState,
};
use crate::manager::DeliveryWorker;
use crate::playback_demand::demand_channel;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::sync::Arc;

pub(super) struct FallbackFixture {
    handle: DeliveryHandle,
    worker: DeliveryWorker,
    post: PostId,
    _root: fallback_environment::TestRoot,
}

impl FallbackFixture {
    pub(super) async fn new() -> Self {
        let root = fallback_environment::test_root();
        let store = fallback_environment::store(&root.0);
        let (handle, commands) = command_channel();
        let (_demand, demand) = demand_channel();
        let config = fallback_environment::config(store, root.0.join("stats.json"));
        let resources = crate::manager::resource_control::ResourceControl::bootstrap(
            &config,
            tokio::time::Instant::now(),
        );
        assert!(config
            .requests
            .install_resource_observer(Arc::new(resources.clone())));
        let mut worker = DeliveryWorker::create(config, commands, demand, resources).await;
        let post = PostId::new("malformed");
        let (focus, candidate) = fallback_environment::focus_candidate(post.clone());
        worker.state.apply_focus(focus, 1);
        worker.state.apply_candidate(candidate);
        worker.bind_representations().await;
        Self {
            handle,
            worker,
            post,
            _root: root,
        }
    }

    pub(super) async fn report(&mut self, sequence: u64, failure: Option<&str>) {
        let state = failure.map_or(PlayerPreparationState::Initializing, |_| {
            PlayerPreparationState::Failed
        });
        let binding = self
            .worker
            .state
            .catalog()
            .binding(&self.post)
            .expect("binding");
        let authority = PlayerPreparationAuthority::try_new(
            self.post.clone(),
            binding,
            ContentRevision::default(),
            "asset",
        )
        .expect("authority");
        let attempt = PlayerPreparationAttempt::try_new(7, 1, 1).expect("attempt");
        let observation =
            PlayerPreparationObservation::try_new(state, failure.map(str::to_owned), sequence)
                .expect("observation");
        let report = PlayerPreparationReport::try_new(authority, attempt, sequence, observation)
            .expect("report");
        assert_eq!(
            self.handle.report_player_preparation(report),
            PlayerPreparationIngress::Accepted
        );
        let envelope = self
            .worker
            .commands
            .try_player_preparation_envelope()
            .expect("queued report");
        self.worker
            .apply_player_preparation_feedback(envelope)
            .await;
    }

    pub(super) fn selected_fallback(&self) -> bool {
        self.worker
            .state
            .catalog()
            .lookup(&self.post)
            .expect("catalog entry")
            .meta
            .urls[0]
            == "https://fallback.example/video.mp4"
    }
}
