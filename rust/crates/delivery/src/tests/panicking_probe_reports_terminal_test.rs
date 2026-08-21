use crate::debug::network::NetworkThrottle;
use crate::manager::response_open;
use crate::manager::traffic;
use crate::manager::transfers::{spawn_probe, InternalEvent, TransferContext, TransferEvent};
use crate::tests::support::temp_directory;
use ghostr_engine::PostId;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

struct PanicClient;

impl MediaHttpRequests for PanicClient {
    fn get(&self, _url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        panic!("fixture probe panic")
    }
}

#[tokio::test]
async fn panicking_probe_reports_a_terminal_event() {
    let root = temp_directory("panicking-probe");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(4),
    ));
    let (events_sender, mut events) = mpsc::unbounded_channel();
    let (publisher, _traffic) = traffic::channel(events_sender.clone(), 4);
    let (responses, _response_receiver) = response_open::channel(Duration::from_secs(1));
    let ctx = TransferContext {
        client: Arc::new(PanicClient),
        store,
        events: events_sender,
        responses,
        timeouts: TransferTimeouts::default(),
        network: NetworkThrottle::new(),
        traffic: publisher,
    };

    spawn_probe(
        ctx,
        PostId::new("post"),
        "https://panic.example/video".to_owned(),
    );

    let event = tokio::time::timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("probe completion deadline")
        .expect("probe terminal event");
    let InternalEvent::Transfer(TransferEvent::ProbeDone(done)) = event else {
        panic!("probe terminal event")
    };
    assert_eq!(done.post, PostId::new("post"));
    assert_eq!(done.url, "https://panic.example/video");
    assert!(done
        .outcome
        .unwrap_err()
        .to_string()
        .contains("task failed"));
    std::fs::remove_dir_all(root).unwrap();
}
