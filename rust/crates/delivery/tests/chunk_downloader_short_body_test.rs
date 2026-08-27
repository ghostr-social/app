mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn a_body_shorter_than_its_advertised_range_is_rejected() {
    let url = range_fixture::short::serve_short_partial().await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-short");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(4, 12)),
        attempt_profile: range_fixture::range_profile(8),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result = download_chunk_throttled(
        &spec,
        &sink,
        range_fixture::context(&mut stats, &token, &range_fixture::network()),
    )
    .await;

    assert!(result.is_err(), "short 206 must not count as success");
    assert!(store
        .present_ranges("clip")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert_range_failure(&stats, &url);
    std::fs::remove_dir_all(root).ok();
}

fn assert_range_failure(stats: &HostStats, url: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid test fixture")
        .as_millis() as u64;
    let context = OriginContext::new(RequestMethod::RangeGet, 8, MediaClass::ProgressiveMp4)
        .with_observed_at_ms(now);
    let range = stats
        .origin_model()
        .estimate(&OriginQuery::new(url, context), now, DecisionMode::Normal)
        .range_compliance
        .expect("range posterior");
    assert!(range.mean < 0.6, "short 206 must lower range compliance");
}
