use super::asset;
use super::support::{client, oversized_headers};
use ghostr_engine::adaptive::PreemptionAuthority;
use url::Url;

#[tokio::test]
async fn hls_fetch_rejects_oversized_response_headers() {
    let (raw_url, server) = oversized_headers().await;
    let url = Url::parse(&raw_url).expect("URL");

    let requests = client();
    let error = match asset(&requests, &url, PreemptionAuthority::Transition).await {
        Ok(_) => panic!("oversized headers must be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("headers exceed byte limit"));
    server.await.expect("server");
}
