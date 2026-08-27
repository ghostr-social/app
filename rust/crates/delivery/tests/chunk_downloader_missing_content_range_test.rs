mod delivery_fixture;
mod range_fixture;
mod raw_http;

use delivery_fixture::{media_client, temp_directory};
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{
    ChunkSink, ChunkSpec, ResponseObservation, ResponseRejection,
};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use range_fixture::{download_chunk_with_traffic, ObservationTraffic};
use raw_http::spawn_raw_server;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[tokio::test]
async fn chunk_downloader_rejects_partial_content_without_a_content_range() {
    let response =
        b"HTTP/1.1 206 Partial Content\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, request) = spawn_raw_server(response).await;
    let root = temp_directory("ghostr-missing-content-range");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &media_client(),
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 5)),
        attempt_profile: range_fixture::range_profile(5),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let mut traffic = ObservationTraffic::default();
    let outcome = download_chunk_with_traffic(
        &spec,
        &sink,
        range_fixture::context(&mut stats, &token, &NetworkThrottle::new()),
        &mut traffic,
    )
    .await;

    assert!(outcome.is_err(), "malformed partial response must fail");
    assert_eq!(
        traffic.observation(),
        Some(ResponseObservation::Rejected(ResponseRejection::Semantics))
    );
    assert!(store
        .present_ranges("clip")
        .await
        .expect("ranges")
        .is_empty());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid test fixture")
        .as_millis() as u64;
    let query = OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::RangeGet, 5, MediaClass::ProgressiveMp4)
            .with_observed_at_ms(now),
    );
    let range = stats
        .origin_model()
        .estimate(&query, now, DecisionMode::Normal)
        .range_compliance
        .expect("range posterior");
    assert!(
        range.mean < 0.6,
        "malformed 206 must lower range compliance"
    );
    request.await.expect("upstream request");
    std::fs::remove_dir_all(root).ok();
}
