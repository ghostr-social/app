use super::super::asset;
use super::support::{client, oversized_headers};
use url::Url;

#[tokio::test]
async fn hls_fetch_rejects_oversized_response_headers() {
    let (raw_url, server) = oversized_headers().await;
    let url = Url::parse(&raw_url).expect("URL");

    let error = match asset(client().as_ref(), &url).await {
        Ok(_) => panic!("oversized headers must be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("headers exceed byte limit"));
    server.await.expect("server");
}
