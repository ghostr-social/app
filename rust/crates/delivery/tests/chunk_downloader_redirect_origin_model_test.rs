mod range_fixture;
mod raw_http;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn redirected_range_updates_the_requested_routes_model() {
    let target = range_fixture::ranged::serve_ranged(range_fixture::body())
        .await
        .replacen("127.0.0.1", "localhost", 1);
    let (start, redirect_request) = raw_http::spawn_redirect(&target).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-redirect-origin-model");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &start,
        request: range_fixture::range_request(ByteRange::new(0, 16)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    download_chunk_throttled(
        &spec,
        &sink,
        range_fixture::context(&mut stats, &token, &range_fixture::network()),
    )
    .await
    .expect("redirected range delivery");

    let estimate =
        stats
            .origin_model()
            .estimate(&query(start.clone()), unix_time_ms(), DecisionMode::Normal);
    assert!(
        estimate.effective_samples > 0.9,
        "requested route only received global fallback evidence: {estimate:?}"
    );
    redirect_request.await.unwrap();
    let _ = std::fs::remove_dir_all(root);
}

fn query(url: String) -> OriginQuery {
    OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::RangeGet, 16, MediaClass::ProgressiveMp4)
            .with_network(NetworkClass::Unavailable)
            .with_concurrency(1)
            .with_observed_at_ms(unix_time_ms()),
    )
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
