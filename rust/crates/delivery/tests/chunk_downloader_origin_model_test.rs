mod range_fixture;

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
async fn real_range_outcome_updates_contextual_origin_model() {
    let url = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-origin-model");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
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
    .expect("range delivery");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid test fixture")
        .as_millis() as u64;
    let query = OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::RangeGet, 16, MediaClass::ProgressiveMp4)
            .with_network(NetworkClass::Unavailable)
            .with_concurrency(1)
            .with_observed_at_ms(now),
    );
    let estimate = stats
        .origin_model()
        .estimate(&query, now, DecisionMode::Normal);
    assert!(estimate.effective_samples > 0.0);
    assert!(estimate.range_compliance.expect("range posterior").mean > 0.5);
    assert!(estimate.ttfb_ms.p50 > 0);
    assert!(estimate.throughput_bps.p50 > 0);
    let _ = std::fs::remove_dir_all(root);
}
