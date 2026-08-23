use crate::cache::client_with_event_cache;
use crate::execution::relay_executor::RelayPlanExecutor;
use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo, RelayReadResult};
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::tests::outbox_support::{empty_directory, BOOTSTRAP_RELAY};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, Filter, Kind};
use std::sync::{Arc, Mutex};

pub(crate) struct BlossomIo {
    video: Event,
    servers: Event,
    pub(crate) filters: Mutex<Vec<Filter>>,
}

impl BlossomIo {
    pub(crate) fn new(video: Event, servers: Event) -> Arc<Self> {
        Arc::new(Self {
            video,
            servers,
            filters: Mutex::new(Vec::new()),
        })
    }
}

impl RelayIo for BlossomIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, RelayReadResult> {
        Box::pin(async move {
            self.filters
                .lock()
                .expect("filters")
                .push(request.filter.clone());
            if has_kind(&request.filter, Kind::Custom(10063)) {
                return Ok(RelayReadResult::complete(vec![self.servers.clone()]));
            }
            if has_kind(&request.filter, Kind::TextNote) && request.filter.search.is_none() {
                return Ok(RelayReadResult::complete(vec![self.video.clone()]));
            }
            Ok(RelayReadResult::complete(Vec::new()))
        })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

pub(crate) fn executor(io: Arc<BlossomIo>) -> RelayPlanExecutor {
    let configuration = RelayPoolConfiguration {
        read_relays: vec![BOOTSTRAP_RELAY.to_owned()],
        search_relays: vec![BOOTSTRAP_RELAY.to_owned()],
    };
    let owner = Arc::new(RelayPoolOwner::with_io(
        Arc::new(client_with_event_cache()),
        configuration,
        io,
    ));
    RelayPlanExecutor::with_owner(
        owner,
        vec![BOOTSTRAP_RELAY.to_owned()],
        empty_directory(),
        DataUsageLevel::Balanced,
    )
}

pub(crate) fn has_kind(filter: &Filter, kind: Kind) -> bool {
    filter
        .kinds
        .as_ref()
        .is_some_and(|kinds| kinds.contains(&kind))
}
