mod range_fixture;
mod raw_http;
#[path = "chunk_downloader_header_timeout_capacity_test/support.rs"]
mod support;

use core::time::Duration;
use ghostr_delivery::manager::failure::{classify, FailureClass};
use ghostr_engine::RequestAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use support::{open_healthy, DownloadFixture};

#[tokio::test]
async fn a_header_timeout_is_transient_and_returns_single_request_capacity() {
    let stalled = raw_http::spawn_stalled_headers().await;
    let healthy = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let requests = MediaRequestExecutor::new(
        range_fixture::raw_media_client(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let fixture = DownloadFixture::new(&requests);

    let error = fixture
        .download(&stalled.url)
        .await
        .expect_err("stalled headers must reach their local deadline");
    assert_eq!(classify(&error), FailureClass::Transient);
    let stalled_authority = RequestAuthority::from_url(&stalled.url).expect("loopback authority");
    assert_eq!(requests.active_for(&stalled_authority), 0);

    let recovered = tokio::time::timeout(Duration::from_secs(1), open_healthy(&requests, &healthy))
        .await
        .expect("healthy origin response deadline")
        .expect("released executor capacity admits a healthy request");
    assert!(recovered.status().is_success());

    stalled.requests.await.expect("stalled request task");
}
