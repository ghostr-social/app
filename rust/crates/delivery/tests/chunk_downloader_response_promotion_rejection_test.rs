mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::adaptive::{PromotionGrant, RetrievalRequest};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;

#[tokio::test]
async fn rejected_promotion_learns_range_semantics_but_not_open_body_failure() {
    let bytes = range_fixture::body();
    let url = range_fixture::ranged::serve_range_blind(bytes.clone()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-rejected-response-promotion");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let request = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 8),
        promotion: Some(PromotionGrant {
            maximum_bytes: 64,
            valid_until_ms: u64::MAX,
        }),
    };
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request,
        attempt_profile: range_fixture::range_profile(8),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let mut traffic = range_fixture::download::rejection_traffic();
    let result = range_fixture::download_chunk_with_traffic(
        &spec,
        &sink,
        range_fixture::context(&mut stats, &token, &range_fixture::network()),
        &mut traffic,
    )
    .await
    .expect("policy rejection is cancellation");

    assert!(result.cancelled);
    assert_eq!(result.bytes_written, 0);
    let model = stats.origin_model();
    let range = model.estimate(&query(&url, 8), now(), DecisionMode::Normal);
    let body = model.estimate_open_body(
        &query(&url, bytes.len() as u64),
        now(),
        DecisionMode::Normal,
    );
    assert!(range.range_compliance.is_some_and(|value| value.mean < 0.6));
    assert_eq!(body.effective_samples, 0.0);
    let _ = std::fs::remove_dir_all(root);
}

fn query(url: &str, bytes: u64) -> OriginQuery {
    let context = OriginContext::new(RequestMethod::RangeGet, bytes, MediaClass::ProgressiveMp4)
        .with_network(NetworkClass::Unavailable)
        .with_observed_at_ms(now());
    OriginQuery::new(url, context)
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_millis() as u64
}
