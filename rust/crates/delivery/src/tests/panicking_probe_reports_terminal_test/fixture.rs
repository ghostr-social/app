use crate::debug::network::NetworkThrottle;
use crate::delivery_events::{CommandReceiver, DecisionClaim, DeliveryHandle};
use crate::manager::response_open;
use crate::manager::traffic;
use crate::manager::transfers::TransferContext;
use crate::tests::decision_log_fixture::{head_identity, selected_head};
use crate::tests::support::temp_directory;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub(super) struct TrackedHead {
    pub handle: DeliveryHandle,
    pub commands: CommandReceiver,
    pub sequence: u64,
    pub identity: ghostr_engine::representation::TransferIdentity,
    pub claim: DecisionClaim,
}

struct PanicClient;

impl MediaHttpRequests for PanicClient {
    fn get(&self, _url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        panic!("fixture probe panic")
    }
}

pub(super) fn tracked_head() -> TrackedHead {
    let (handle, commands) = crate::delivery_events::command_channel();
    let (sequence, token) = selected_head(&handle, &commands);
    let identity = head_identity();
    let claim = commands
        .claim_decision(token, &identity, 100)
        .unwrap_or_else(|_| panic!("HEAD claim"));
    TrackedHead {
        handle,
        commands,
        sequence,
        identity,
        claim,
    }
}

pub(super) fn context() -> (TransferContext, PathBuf) {
    let root = temp_directory("panicking-probe");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(4),
    ));
    let (events, _) = tokio::sync::mpsc::unbounded_channel();
    let (traffic, _) = traffic::channel(events.clone(), 4);
    let (responses, _) = response_open::channel(Duration::from_secs(1));
    let context = TransferContext {
        client: Arc::new(PanicClient),
        store,
        events,
        responses,
        timeouts: TransferTimeouts::default(),
        network: NetworkThrottle::new(),
        traffic,
    };
    (context, root)
}
