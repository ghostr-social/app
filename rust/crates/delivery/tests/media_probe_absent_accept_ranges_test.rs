mod delivery_fixture;
mod probe_fixture;
mod raw_http;

use delivery_fixture::media_client;
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use probe_fixture::probe;
use raw_http::spawn_raw_server;

#[tokio::test]
async fn absent_accept_ranges_remains_unknown_without_a_probe_get() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\n\r\n";
    let (url, request) = spawn_raw_server(response).await;
    let mut stats = HostStats::new();

    let requests = media_client();
    let result = probe(&requests, &url, TransferTimeouts::default(), &mut stats)
        .await
        .expect("HEAD metadata");
    let request =
        String::from_utf8(request.await.expect("valid test fixture")).expect("valid test fixture");

    assert_eq!(result.content_length, Some(16));
    assert_eq!(result.accept_ranges, None);
    assert!(request.starts_with("HEAD "));
}
