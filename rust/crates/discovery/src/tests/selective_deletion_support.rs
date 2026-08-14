use crate::cache::client_with_event_cache;
use crate::execution::relay_executor::RelayPlanExecutor;
use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::tests::outbox_support::{empty_directory, BOOTSTRAP_RELAY};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, Kind};
use std::sync::Arc;

pub(crate) struct SelectiveDeletionIo {
    wrappers: Vec<Event>,
    failing_relay: String,
}

impl SelectiveDeletionIo {
    pub(crate) fn new(wrappers: Vec<Event>, failing_relay: &str) -> Arc<Self> {
        Arc::new(Self {
            wrappers,
            failing_relay: failing_relay.to_owned(),
        })
    }
}

impl RelayIo for SelectiveDeletionIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async move {
            let deletion = request
                .filter
                .kinds
                .as_ref()
                .is_some_and(|kinds| kinds.contains(&Kind::EventDeletion));
            if deletion && request.relays.contains(&self.failing_relay) {
                return Err(anyhow::anyhow!("selected deletion relay offline"));
            }
            Ok(self
                .wrappers
                .iter()
                .filter(|event| request.filter.match_event(event))
                .cloned()
                .collect())
        })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

pub(crate) fn selective_deletion_executor(io: Arc<SelectiveDeletionIo>) -> RelayPlanExecutor {
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
