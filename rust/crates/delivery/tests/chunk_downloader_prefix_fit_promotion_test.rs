mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec, ResponseObservation};
use ghostr_engine::adaptive::{PromotionGrant, RetrievalRequest, WholeFetchReason};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::{download_chunk_with_traffic, ObservationTraffic};

#[tokio::test]
async fn a_whole_body_fitting_the_prefix_uses_existing_authority() {
    let bytes = range_fixture::body();
    let url = range_fixture::ranged::serve_range_blind(bytes.clone()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-prefix-fit-200");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let request = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 32),
        promotion: Some(PromotionGrant {
            maximum_bytes: 64,
            valid_until_ms: u64::MAX,
        }),
    };
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request,
        attempt_profile: range_fixture::range_profile(32),
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
    .expect("covered whole response");

    let ResponseObservation::Body {
        request: RetrievalRequest::FetchWhole { reason, .. },
        promoted,
        ..
    } = traffic.observation().expect("observed response")
    else {
        panic!("expected a bounded whole response");
    };
    assert_eq!(reason, WholeFetchReason::PlannedCompletion);
    assert!(!promoted);
    assert!(!result.cancelled);
    assert_eq!(result.bytes_written, bytes.len() as u64);
    let _ = std::fs::remove_dir_all(root);
}
