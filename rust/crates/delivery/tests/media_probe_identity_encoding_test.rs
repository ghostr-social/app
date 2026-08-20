mod delivery_fixture;
mod raw_http;

use delivery_fixture::media_client;
use ghostr_delivery::probe::media::probe;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use raw_http::spawn_raw_server;

const CODED_HEAD: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: video/mp4\r\n\
Content-Length: 16\r\n\
ETag: \"coded\"\r\n\
Content-Encoding: identity\r\n\
Content-Encoding: gzip\r\n\r\n";

#[tokio::test]
async fn media_probe_requires_one_identity_encoded_representation() {
    let (url, request) = spawn_raw_server(CODED_HEAD).await;
    let mut stats = HostStats::new();

    let outcome = probe(
        &media_client(),
        &url,
        TransferTimeouts::default(),
        &mut stats,
    )
    .await;
    let request = String::from_utf8(request.await.expect("origin request")).expect("HTTP request");

    assert_eq!(accept_encodings(&request), ["identity"]);
    let error = outcome.expect_err("coded HEAD facts must be rejected");
    assert!(error.to_string().contains("HEAD response is encoded"));
    assert!(stats.failure_ratio(&host_of(&url).expect("origin host")) > 0.0);
}

fn accept_encodings(request: &str) -> Vec<&str> {
    request
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.eq_ignore_ascii_case("accept-encoding"))
        .map(|(_, value)| value.trim())
        .collect()
}
