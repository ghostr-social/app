mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginAttemptProfile, OriginContext, OriginQuery,
    OriginRequestProfile, RequestMethod,
};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn a_real_range_terminal_observation_keeps_its_planned_prefix_context() {
    let url = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-prefix-context");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 16)),
        attempt_profile: OriginAttemptProfile::new(OriginRequestProfile::new(
            RequestMethod::PrefixGet,
            16,
            MediaClass::Unknown,
        )),
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
    .expect("prefix delivery");

    let now = unix_time_ms();
    let query = OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::PrefixGet, 16, MediaClass::Unknown)
            .with_network(NetworkClass::Unavailable)
            .with_concurrency(1)
            .with_observed_at_ms(now),
    );
    assert!(
        stats
            .origin_model()
            .estimate(&query, now, DecisionMode::Normal)
            .effective_samples
            > 0.9
    );
    let _ = std::fs::remove_dir_all(root);
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("valid clock")
        .as_millis() as u64
}
