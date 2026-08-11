use super::progressive::ProgressiveHarness;
use super::progressive_request::capability_request;
use axum::body::Body;
use axum::http::Request;

impl ProgressiveHarness {
    pub async fn video_request(&self, id: &str, range: Option<&str>) -> Request<Body> {
        let capability = self.capabilities.issue(id).await;
        capability_request(id, capability.as_str(), range)
    }
}
