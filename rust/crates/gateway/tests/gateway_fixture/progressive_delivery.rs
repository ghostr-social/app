use super::delivery::{start_delivery, start_delivery_with_tuning, DeliveryFixture};
use super::media_client;
use super::progressive_journey_trace::ProgressiveJourneyTrace;
#[cfg(feature = "video-debug-web")]
use ghostr_delivery::debug::feed::DebugFeed;
use ghostr_delivery::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem};
use ghostr_delivery::manager::DeliveryTuning;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_gateway::progressive::route::{ProgressiveState, ProgressiveTiming};
use ghostr_gateway::router::configured_router_with_progressive;
use std::sync::Arc;

pub struct ProgressiveDeliveryHarness {
    pub delivery: DeliveryFixture,
    pub router: axum::Router,
    pub trace: ProgressiveJourneyTrace,
    capabilities: ProgressiveCapabilities,
}

impl ProgressiveDeliveryHarness {
    pub fn start(prefix: &str) -> Self {
        Self::from_delivery(start_delivery(prefix))
    }

    pub fn start_with_tuning(prefix: &str, tuning: DeliveryTuning) -> Self {
        Self::from_delivery(start_delivery_with_tuning(prefix, tuning))
    }

    fn from_delivery(delivery: DeliveryFixture) -> Self {
        let trace = ProgressiveJourneyTrace::default();
        let capabilities = ProgressiveCapabilities::production();
        let state = Arc::new(ProgressiveState {
            store: delivery.store.clone(),
            demand: delivery.demand.clone(),
            cache: delivery.cache.clone(),
            network: delivery.network.clone(),
            timing: ProgressiveTiming::default(),
            capabilities: capabilities.clone(),
            #[cfg(feature = "video-debug-web")]
            debug_feed: DebugFeed::new(delivery.handle.clone(), Vec::new()),
        });
        let router =
            configured_router_with_progressive(HlsSessions::production(), media_client(), state);
        Self {
            delivery,
            router,
            trace,
            capabilities,
        }
    }

    pub fn focus(&self, items: Vec<FocusItem>, current_index: usize) {
        let focus = DeliveryFocus::compatibility(items, current_index, 0);
        self.trace.record_focus(&focus);
        self.delivery.handle.update_focus(focus);
    }

    pub fn observe(&self, playback: DeliveryPlayback) {
        self.trace.record_observation(playback.clone());
        self.delivery.handle.report_playback(playback);
    }

    pub fn first_frame(&self, post: &str) {
        self.trace
            .record_first_frame(ghostr_engine::PostId::new(post));
    }

    pub fn cancel(&self, post: &str) {
        self.trace
            .record_cancellation(ghostr_engine::PostId::new(post));
    }

    pub async fn wait_until_registered(&self, post: &str) {
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while !self.delivery.cache.contains(post) {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("progressive cache registration");
    }

    pub async fn request(&self, post: &str, range: &str) -> axum::http::Request<axum::body::Body> {
        let capability = self.capabilities.issue(post).await;
        let uri = format!("/video.mp4?id={post}&cap={}", capability.as_str());
        axum::http::Request::builder()
            .uri(uri)
            .header(axum::http::header::RANGE, range)
            .body(axum::body::Body::empty())
            .expect("progressive request")
    }
}

impl Drop for ProgressiveDeliveryHarness {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.delivery.root).ok();
    }
}
