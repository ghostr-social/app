use crate::cache::client_with_event_cache;
use crate::execution::relay_executor::RelayPlanExecutor;
use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::tests::outbox_support::{empty_directory, BOOTSTRAP_RELAY};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, Filter};
use std::sync::{Arc, Mutex};

pub(crate) struct DeletionIo {
    wrapper: Event,
    deletion: Event,
    pub(crate) filters: Mutex<Vec<Filter>>,
}

impl DeletionIo {
    pub(crate) fn new(wrapper: Event, deletion: Event) -> Arc<Self> {
        Arc::new(Self {
            wrapper,
            deletion,
            filters: Mutex::new(Vec::new()),
        })
    }
}

impl RelayIo for DeletionIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async move {
            self.filters
                .lock()
                .expect("filters")
                .push(request.filter.clone());
            if request.filter.match_event(&self.wrapper) {
                return Ok(vec![self.wrapper.clone()]);
            }
            if request.filter.match_event(&self.deletion) {
                return Ok(vec![self.deletion.clone()]);
            }
            Ok(Vec::new())
        })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

pub(crate) fn executor(io: Arc<DeletionIo>) -> RelayPlanExecutor {
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
