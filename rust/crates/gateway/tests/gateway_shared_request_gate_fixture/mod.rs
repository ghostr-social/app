use super::gateway_fixture;
use gateway_fixture::request_gate_origin::{ManifestOrigin, VideoOrigin};
use ghostr_delivery::playback_demand::DemandConsumer;
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::runtime::GatewayRuntime;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

mod setup;

pub(super) struct SharedGateScenario {
    root: PathBuf,
    runtime: GatewayRuntime,
    _demand: DemandConsumer,
    video: VideoOrigin,
    manifest: ManifestOrigin,
}

impl SharedGateScenario {
    pub(super) async fn start() -> Self {
        let root = gateway_fixture::temp_directory("shared-request-gate");
        let video = VideoOrigin::start(32).await;
        let manifest = ManifestOrigin::start().await;
        let (_endpoint, runtime, _updates) = GatewayRuntime::start_debug(
            setup::configuration(root.clone()),
            Arc::new(client_with_event_cache()),
        )
        .await
        .expect("valid test fixture");
        runtime.delivery().update_focus(setup::focus(&video.url));
        let demand = setup::demand(&runtime).await;
        Self {
            root,
            runtime,
            _demand: demand,
            video,
            manifest,
        }
    }

    pub(super) async fn exercise(mut self) {
        let active = tokio::time::timeout(Duration::from_secs(1), self.video.next())
            .await
            .expect("progressive body starts");
        let session = self
            .runtime
            .acquire_hls(vec![self.manifest.url.clone()])
            .await
            .expect("valid test fixture");
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("valid test fixture");
        let hls = tokio::spawn(async move {
            client
                .get(session.playback_url)
                .send()
                .await
                .expect("valid test fixture")
        });
        self.manifest.expect_quiet().await;
        active.finish().await;
        tokio::time::timeout(Duration::from_secs(1), self.manifest.next())
            .await
            .expect("HLS starts after release");
        assert!(hls.await.expect("valid test fixture").status().is_success());
        std::fs::remove_dir_all(self.root).ok();
    }
}
