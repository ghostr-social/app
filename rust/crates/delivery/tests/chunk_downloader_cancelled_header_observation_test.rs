mod range_fixture;

use ghostr_delivery::chunk::cancel::{cancel_pair, CancelHandle};
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkSink, ChunkSpec, DownloadTraffic, OpenedResponse,
    ResponseAdmission,
};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{future::Future, pin::Pin};

#[tokio::test]
async fn headers_are_observed_once_when_cancellation_wins_authorization() {
    let url = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-cancelled-headers");
    let store = range_fixture::store(root.clone());
    let (handle, token) = cancel_pair();
    let mut traffic = CancelAtHeaders {
        handle,
        observed: 0,
        callback_ms: 0,
        boundary_ms: 0,
        authorized: 0,
    };
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 8)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: Default::default(),
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
    .unwrap();

    assert!(result.cancelled);
    assert_eq!(traffic.observed, 1);
    assert_eq!(traffic.authorized, 0);
    assert!(traffic.boundary_ms <= traffic.callback_ms);
    std::fs::remove_dir_all(root).ok();
}

struct CancelAtHeaders {
    handle: CancelHandle,
    observed: usize,
    callback_ms: u64,
    boundary_ms: u64,
    authorized: usize,
}

impl DownloadTraffic for CancelAtHeaders {
    fn opened(&mut self, _: Duration) {
        self.callback_ms = now_ms();
        std::thread::sleep(Duration::from_millis(20));
        self.handle.cancel();
    }

    fn wrote(&mut self, _: u64) {}
    fn response_observed(&mut self, response: OpenedResponse) {
        self.observed += 1;
        self.boundary_ms = response.evidence().observed.observed_at_ms;
    }

    fn authorize_response<'a>(
        &'a mut self,
        _: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        Box::pin(std::future::poll_fn(move |_| {
            self.authorized += 1;
            std::task::Poll::Ready(Ok(ResponseAdmission::Proceed))
        }))
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
