use anyhow::Result;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;

struct RewritingClient(Client);

impl MediaHttpRequests for RewritingClient {
    fn get(&self, _raw_url: &str) -> Result<RequestBuilder> {
        Ok(self.0.get("https://other.example/video"))
    }
}

#[tokio::test]
async fn adapter_cannot_execute_a_different_authority_under_the_raw_url_lease() {
    let requests = MediaRequestExecutor::new(
        Arc::new(RewritingClient(Client::new())),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );

    let result = requests
        .get(
            "https://media.example/video",
            PreemptionAuthority::Transition,
        )
        .unwrap()
        .admit()
        .await;

    assert!(
        result.is_err(),
        "rewritten request authority must fail closed"
    );
}
