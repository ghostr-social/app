mod range_fixture;

use core::time::Duration;
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkSink, ChunkSpec, DownloadTraffic, OpenedResponse,
};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use tokio::sync::oneshot;

#[tokio::test]
async fn ignored_range_reports_complete_header_evidence() {
    let url = range_fixture::ranged::serve_range_blind(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-ignored-headers");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let (sender, observed) = oneshot::channel();
    let mut traffic = HeaderTraffic(Some(sender));
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(8, 16)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let network = range_fixture::network();
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
    .result
    .expect("ignored response");

    let response = observed.await.expect("header observation");
    assert!(result.range_ignored);
    assert_eq!(response.evidence().final_url, url);
    assert_eq!(
        response.evidence().content_type.as_deref(),
        Some("video/mp4")
    );
    assert_eq!(
        response.evidence().validator,
        EvidenceValidator::strong_etag("\"fixture-ranged\"")
    );
    std::fs::remove_dir_all(root).ok();
}

struct HeaderTraffic(Option<oneshot::Sender<OpenedResponse>>);

impl DownloadTraffic for HeaderTraffic {
    fn opened(&mut self, _: Duration) {}
    fn wrote(&mut self, _: u64) {}
    fn response_observed(&mut self, response: OpenedResponse) {
        self.0
            .take()
            .expect("valid test fixture")
            .send(response)
            .ok();
    }
}
