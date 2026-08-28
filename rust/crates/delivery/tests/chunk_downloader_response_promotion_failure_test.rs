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
async fn an_accepted_promoted_body_timeout_lowers_continuation_success() {
    let url = range_fixture::promoted_stall::serve(vec![7; 1_024], 100_000).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-promoted-body-timeout");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let request = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 32_000),
        promotion: Some(PromotionGrant {
            maximum_bytes: 200_000,
            valid_until_ms: u64::MAX,
        }),
    };
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request,
        attempt_profile: range_fixture::range_profile(32_000),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: short_timeouts(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let result = range_fixture::download_chunk_throttled(
        &spec,
        &sink,
        range_fixture::context(&mut stats, &token, &range_fixture::network()),
    )
    .await;

    assert!(result
        .expect_err("body must time out")
        .to_string()
        .contains("timed out"));
    let estimate = stats.origin_model().estimate_open_body(
        &query(&url, RequestMethod::RangeGet, 100_000),
        now(),
        DecisionMode::Normal,
    );
    assert!(estimate.effective_samples > 0.0);
    assert!(estimate.success.mean < 0.9);
    assert!(range_noncompliance(&stats, &url));
    let _ = std::fs::remove_dir_all(root);
}

fn range_noncompliance(stats: &HostStats, url: &str) -> bool {
    let estimate = stats.origin_model().estimate(
        &query(url, RequestMethod::RangeGet, 32_000),
        now(),
        DecisionMode::Normal,
    );
    estimate
        .range_compliance
        .is_some_and(|value| value.mean < 0.6)
}

fn query(url: &str, method: RequestMethod, bytes: u64) -> OriginQuery {
    let context = OriginContext::new(method, bytes, MediaClass::ProgressiveMp4)
        .with_network(NetworkClass::Unavailable)
        .with_observed_at_ms(now());
    OriginQuery::new(url, context)
}

fn short_timeouts() -> TransferTimeouts {
    let short = core::time::Duration::from_millis(100);
    TransferTimeouts {
        admission: short,
        headers: short,
        idle: short,
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_millis() as u64
}
