mod range_fixture;
mod raw_http;

#[path = "chunk_downloader_duplicate_content_range_test/header_traffic.rs"]
mod header_traffic;
use header_traffic::HeaderTraffic;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkSink, ChunkSpec, ResponseObservation,
    ResponseRejection,
};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;

const DUPLICATE_RANGE: &[u8] = b"HTTP/1.1 206 Partial Content\r\n\
Content-Type: video/mp4\r\n\
Content-Length: 8\r\n\
Content-Range: bytes 8-15/16\r\n\
Content-Range: bytes 8-15/32\r\n\
ETag: \"fixture-media\"\r\n\
Connection: close\r\n\r\n89abcdef";

#[tokio::test]
async fn duplicate_content_range_cannot_extend_a_sparse_generation() {
    let (url, request) = raw_http::spawn_raw_server(DUPLICATE_RANGE).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("duplicate-content-range");
    let store = range_fixture::store(root.clone());
    store
        .write_range("clip", 0, b"01234567")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("clip", 16)
        .await
        .expect("valid test fixture");
    let generation =
        SourceGeneration::try_new(&url, "\"fixture-media\"", 16).expect("valid test fixture");
    let (_handle, token) = cancel_pair();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(8, 16)),
        attempt_profile: range_fixture::range_profile(8),
        continuation: Some(&generation),
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let mut stats = HostStats::new();
    let network = range_fixture::network();
    let mut traffic = HeaderTraffic::default();
    let result = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut stats,
            cancel: &token,
            network: &network,
            traffic: &mut traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    )
    .await
    .result;

    assert!(result.is_err(), "ambiguous 206 must fail before storage");
    let headers = traffic.observed.expect("arrived headers");
    assert_eq!(
        headers.observation(),
        ResponseObservation::Rejected(ResponseRejection::Semantics)
    );
    assert_eq!(
        headers.evidence().validator,
        EvidenceValidator::strong_etag("\"fixture-media\"")
    );
    assert_eq!(
        store
            .present_ranges("clip")
            .await
            .expect("valid test fixture"),
        vec![0..8]
    );
    let request =
        String::from_utf8(request.await.expect("valid test fixture")).expect("valid test fixture");
    assert!(request.contains("range: bytes=8-15"));
    assert!(request.contains("if-range: \"fixture-media\""));
    std::fs::remove_dir_all(root).ok();
}
