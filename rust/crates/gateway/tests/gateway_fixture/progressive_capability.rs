use super::progressive::ProgressiveHarness;
use super::progressive_request::capability_request;
use axum::body::Body;
use axum::http::Request;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilityId;

impl ProgressiveHarness {
    pub async fn video_request(&self, id: &str, range: Option<&str>) -> Request<Body> {
        let capability = self.issue_video_asset(id).await;
        capability_request(id, capability.as_str(), range)
    }

    pub async fn issue_video_asset(&self, id: &str) -> ProgressiveCapabilityId {
        let snapshot = self.store.media_snapshot(id).await.expect("media snapshot");
        self.capabilities
            .issue(&snapshot)
            .await
            .expect("bound media asset")
    }

    pub async fn bind_video(&self, id: &str, source: &str, size: Option<u64>) {
        let mut catalog = Catalog::new();
        let meta = VideoMeta {
            urls: vec![source.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: size,
            duration_ms: Some(1_000),
        };
        let binding = catalog.upsert(PostId::new(id), meta);
        self.store
            .bind_representation(binding)
            .await
            .expect("binding");
    }
}
