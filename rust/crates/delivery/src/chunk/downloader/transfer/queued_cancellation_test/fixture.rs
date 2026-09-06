use super::super::{run, TransferExecution};
use crate::chunk::cancel::{cancel_pair, CancelHandle, CancelToken};
use crate::chunk::downloader::{ChunkResult, ChunkSink};
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{
    AdmittedMediaRequest, MediaRequestExecutor, MediaRequestLimits,
};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;

#[path = "fixture/support.rs"]
mod support;
use support::{expect_pending, spec, IgnoreTraffic, URL};

struct QueuedFixture {
    requests: MediaRequestExecutor,
    held: AdmittedMediaRequest,
    store: PartialRangeStore,
    handle: CancelHandle,
    token: CancelToken,
    network: NetworkThrottle,
}

impl QueuedFixture {
    async fn new() -> Self {
        let raw: Arc<dyn MediaHttpRequests> =
            Arc::new(MediaHttpClient::public().expect("valid test fixture"));
        let requests = MediaRequestExecutor::new(
            raw,
            MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
        );
        let held = requests
            .get(URL, PreemptionAuthority::Transition)
            .expect("valid test fixture")
            .admit()
            .await
            .expect("valid test fixture");
        let root = std::env::temp_dir().join(format!("queued-cancel-{}", std::process::id()));
        let store = PartialRangeStore::with_capacity(
            root,
            Arc::new(tokio::sync::Mutex::new(0)),
            StoreCapacity::system(u64::MAX),
        );
        let (handle, token) = cancel_pair();
        Self {
            requests,
            held,
            store,
            handle,
            token,
            network: NetworkThrottle::new(),
        }
    }

    async fn cancel(self) -> ChunkResult {
        let Self {
            requests,
            held,
            store,
            handle,
            token,
            network,
        } = self;
        let sink = ChunkSink {
            store: &store,
            key: "clip",
        };
        let mut traffic = IgnoreTraffic;
        let spec = spec(&requests);
        let future = run(
            &spec,
            TransferExecution {
                sink: &sink,
                cancel: &token,
                network: Some(&network),
                traffic: &mut traffic,
            },
        );
        tokio::pin!(future);
        expect_pending(&mut future).await;
        handle.cancel();
        let result = future.await.expect("cancelled transfer");
        drop(held);
        result
    }
}

pub(super) async fn cancel_queued_transfer() -> ChunkResult {
    Box::pin(QueuedFixture::new().await.cancel()).await
}
