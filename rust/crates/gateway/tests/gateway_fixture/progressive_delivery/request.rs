use super::ProgressiveDeliveryHarness;
use axum::body::Body;
use axum::http::header::RANGE;
use axum::http::Request;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::time::Duration;

impl ProgressiveDeliveryHarness {
    pub async fn request(&self, post: &str, range: &str) -> Request<Body> {
        let snapshot = tokio::time::timeout(Duration::from_secs(3), self.asset_snapshot(post))
            .await
            .expect("progressive asset authority");
        let capability = self
            .capabilities
            .issue(&snapshot)
            .await
            .expect("bound media asset");
        let uri = format!("/video.mp4?id={post}&cap={}", capability.as_str());
        Request::builder()
            .uri(uri)
            .header(RANGE, range)
            .body(Body::empty())
            .expect("progressive request")
    }

    async fn asset_snapshot(&self, post: &str) -> StoredMediaSnapshot {
        let notify = self.delivery.store.change_notifier();
        loop {
            let changed = notify.notified();
            let snapshot = self
                .delivery
                .store
                .media_snapshot(post)
                .await
                .expect("media snapshot");
            if snapshot.binding().is_some() && snapshot.total_len().is_some() {
                return snapshot;
            }
            changed.await;
        }
    }
}
