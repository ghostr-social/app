use anyhow::Result;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::header::HOST;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;

struct HostOverrideClient(Client);

impl MediaHttpRequests for HostOverrideClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        Ok(self.0.get(raw_url).header(HOST, "other.example"))
    }
}

#[tokio::test]
async fn explicit_host_cannot_bypass_the_gated_request_authority() {
    let requests = MediaRequestExecutor::new(
        Arc::new(HostOverrideClient(Client::new())),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    let request = requests
        .get(
            "https://media.example/video",
            PreemptionAuthority::Transition,
        )
        .unwrap();

    assert!(
        request.admit().await.is_err(),
        "explicit Host must fail closed"
    );
}
