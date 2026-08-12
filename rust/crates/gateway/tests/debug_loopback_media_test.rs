#![cfg(feature = "video-debug-web")]

use axum::{routing::get, Router};
use ghostr_gateway::debug::media::DebugMediaHttpClient;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use tokio::net::TcpListener;

#[tokio::test]
async fn explicit_debug_client_reaches_only_literal_loopback_or_public_media() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let url = format!(
        "http://{}/media.mp4",
        listener.local_addr().expect("address")
    );
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route("/media.mp4", get(|| async { "fixture" })),
        )
        .await
        .expect("serve");
    });
    let client = DebugMediaHttpClient::new().expect("debug media client");

    let response = client
        .get(&url)
        .expect("loopback request")
        .send()
        .await
        .expect("response");

    assert_eq!(response.text().await.expect("body"), "fixture");
    assert!(client.get("http://192.168.1.1/private.mp4").is_err());
    server.abort();
}
