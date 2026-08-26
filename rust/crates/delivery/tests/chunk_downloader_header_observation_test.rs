mod range_fixture;

use core::time::Duration;
use core::{future::Future, pin::Pin};
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkSink, ChunkSpec, DownloadTraffic,
    HttpResponseEvidence, OpenedResponse, ResponseAdmission, ResponseObservation,
};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use tokio::sync::oneshot;

#[tokio::test]
async fn coherent_response_semantics_are_reported_before_body_completion() {
    let (url, _request_started) =
        range_fixture::stall::serve_stalling_signaled(b"ab".to_vec(), 8).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-header-observation");
    let store = range_fixture::store(root.clone());
    let (handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let (sender, observed) = oneshot::channel();
    let mut traffic = HeaderTraffic(Some(sender));
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 8)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let network = range_fixture::network();
    let download = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut stats,
            cancel: &token,
            network: &network,
            traffic: &mut traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    );
    tokio::pin!(download);

    let (response, headers) = tokio::select! {
        result = &mut download => panic!("body ended before header event: {result:?}"),
        result = observed => result.expect("response observation"),
    };
    assert_eq!(
        response,
        ResponseObservation::Partial {
            range: ByteRange::new(0, 8),
            total: Some(8),
        }
    );
    assert_eq!(headers.content_type.as_deref(), Some("video/mp4"));
    assert_eq!(
        headers.validator,
        EvidenceValidator::strong_etag("\"fixture-stall\"")
    );
    handle.cancel();
    assert!(download.await.result.expect("valid test fixture").cancelled);
    std::fs::remove_dir_all(root).ok();
}

struct HeaderTraffic(Option<oneshot::Sender<(ResponseObservation, HttpResponseEvidence)>>);

impl DownloadTraffic for HeaderTraffic {
    fn opened(&mut self, _: Duration) {}

    fn wrote(&mut self, _: u64) {}

    fn authorize_response<'a>(
        &'a mut self,
        response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        let observed = response.observation();
        let evidence = response.evidence().clone();
        self.0
            .take()
            .expect("valid test fixture")
            .send((observed, evidence))
            .ok();
        Box::pin(async { Ok(ResponseAdmission::Proceed) })
    }
}
