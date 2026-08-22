use crate::chunk::cancel::cancel_pair;
use crate::debug::network::NetworkThrottle;
use crate::manager::inflight::ChunkAttempt;
use crate::manager::{response_open, traffic};
use crate::manager::transfers::{
    spawn_chunk, ChunkLaunch, InternalEvent, TransferContext, TransferEvent,
};
use crate::tests::support::temp_directory;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{ActionId, ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::{capacity::StoreCapacity, PartialRangeStore};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

struct PanicClient;
impl MediaHttpRequests for PanicClient {
    fn get(&self, _url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        panic!("fixture transport panic")
    }
}
#[tokio::test]
async fn panicking_transfer_reports_terminal_and_releases_its_reservation() {
    let root = temp_directory("panicking-transfer");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(4),
    ));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://panic.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    let action = store.reserve_action(&identity, 1, 4).await.unwrap();
    let (events_sender, mut events) = mpsc::unbounded_channel();
    let (publisher, _traffic) = traffic::channel(events_sender.clone(), 4);
    let (responses, _response_receiver) = response_open::channel(std::time::Duration::from_secs(1));
    let ctx = TransferContext {
        requests: MediaRequestExecutor::new(
            Arc::new(PanicClient),
            MediaRequestLimits::try_new(1, 1).unwrap(),
        ),
        store: store.clone(),
        events: events_sender,
        responses,
        timeouts: TransferTimeouts::default(),
        network: NetworkThrottle::new(),
        traffic: publisher,
        network_status: crate::delivery_events::DeliveryNetworkStatusReader::new(
            crate::delivery_events::DeliveryNetworkStatus::unavailable(),
        ),
    };
    let attempt = ChunkAttempt::new(
        ChunkId {
            post: PostId::new("post"),
            range: ByteRange::new(0, 4),
        },
        identity.clone(),
        ActionId::new(1),
    );
    let (_cancel, token) = cancel_pair();
    spawn_chunk(ChunkLaunch {
        context: ctx,
        attempt,
        url: identity.source().as_str().to_owned(),
        retrieval: request(),
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        token,
        action,
        network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
    });

    let InternalEvent::Transfer(TransferEvent::ChunkDone(done)) = events.recv().await.unwrap()
    else {
        panic!("chunk terminal event")
    };
    assert!(done.outcome.is_err());
    store.reserve_action(&identity, 2, 4).await.unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

fn request() -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 4),
        promotion: None,
    }
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://panic.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1_000),
    }
}
