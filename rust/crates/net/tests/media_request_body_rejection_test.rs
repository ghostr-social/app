use anyhow::Result;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;

struct BodyClient(Client);

impl MediaHttpRequests for BodyClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        Ok(self.0.get(raw_url).body("hidden upload"))
    }
}

#[tokio::test]
async fn outbound_media_request_body_is_rejected_before_admission() {
    let requests = MediaRequestExecutor::new(
        Arc::new(BodyClient(Client::new())),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let request = requests
        .get(
            "https://media.example/video",
            PreemptionAuthority::Transition,
        )
        .expect("valid test fixture");

    assert!(
        request.admit().await.is_err(),
        "request body must fail closed"
    );
}
