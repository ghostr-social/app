mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec, ResponseObservation};
use ghostr_engine::adaptive::{PromotionGrant, RetrievalRequest};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::{download_chunk_with_traffic, ObservationTraffic};

#[tokio::test]
async fn promoted_200_keeps_range_provenance_and_learns_open_body_success() {
    let bytes = vec![7; 100_000];
    let url = range_fixture::ranged::serve_range_blind(bytes.clone()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-response-promotion-model");
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
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let mut traffic = ObservationTraffic::default();
    let result = download_chunk_with_traffic(
        &spec,
        &sink,
        range_fixture::context(&mut stats, &token, &range_fixture::network()),
        &mut traffic,
    )
    .await
    .expect("promoted response");

    assert!(matches!(
        traffic.observation(),
        Some(ResponseObservation::Body { promoted: true, .. })
    ));
    assert_eq!(result.bytes_written, bytes.len() as u64);
    assert!(range_noncompliance_learned(&stats, &url));
    assert!(open_body_learned(&stats, &url, bytes.len() as u64));
    let _ = std::fs::remove_dir_all(root);
}

fn range_noncompliance_learned(stats: &HostStats, url: &str) -> bool {
    let estimate = stats.origin_model().estimate(
        &query(url, 32_000, RequestMethod::RangeGet),
        now(),
        DecisionMode::Normal,
    );
    estimate
        .range_compliance
        .is_some_and(|range| range.mean < 0.6)
}

fn open_body_learned(stats: &HostStats, url: &str, bytes: u64) -> bool {
    let model = stats.origin_model();
    let range = model.estimate_open_body(
        &query(url, bytes, RequestMethod::RangeGet),
        now(),
        DecisionMode::Normal,
    );
    let full = model.estimate_open_body(
        &query(url, bytes, RequestMethod::FullGet),
        now(),
        DecisionMode::Normal,
    );
    range.effective_samples > 0.0 && full.effective_samples == 0.0
}

fn query(url: &str, bytes: u64, method: RequestMethod) -> OriginQuery {
    let context = OriginContext::new(method, bytes, MediaClass::ProgressiveMp4)
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
