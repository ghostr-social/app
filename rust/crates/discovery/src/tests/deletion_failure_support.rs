use crate::cache::client_with_event_cache;
use crate::execution::relay_executor::RelayPlanExecutor;
use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::tests::outbox_support::{empty_directory, BOOTSTRAP_RELAY};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, Kind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

pub(crate) struct DeletionFailureIo {
    wrapper: Event,
    safe: Event,
    failed: AtomicBool,
    wrapper_reads: AtomicUsize,
}

impl DeletionFailureIo {
    pub(crate) fn new(wrapper: Event, safe: Event) -> Arc<Self> {
        Arc::new(Self {
            wrapper,
            safe,
            failed: AtomicBool::new(false),
            wrapper_reads: AtomicUsize::new(0),
        })
    }
}

impl RelayIo for DeletionFailureIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async move { self.events_for(&request) })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

impl DeletionFailureIo {
    fn events_for(&self, request: &RelayReadIo) -> anyhow::Result<Vec<Event>> {
        if self.should_fail(request) {
            return Err(anyhow::anyhow!("deletion relay offline"));
        }
        Ok([self.safe_event(request), self.wrapper_event(request)]
            .into_iter()
            .flatten()
            .collect())
    }

    fn should_fail(&self, request: &RelayReadIo) -> bool {
        targeted_deletion(request) && !self.failed.swap(true, Ordering::Relaxed)
    }

    fn safe_event(&self, request: &RelayReadIo) -> Option<Event> {
        request
            .filter
            .match_event(&self.safe)
            .then(|| self.safe.clone())
    }

    fn wrapper_event(&self, request: &RelayReadIo) -> Option<Event> {
        if !request.filter.match_event(&self.wrapper) {
            return None;
        }
        (self.wrapper_reads.fetch_add(1, Ordering::Relaxed) == 0).then(|| self.wrapper.clone())
    }
}

fn targeted_deletion(request: &RelayReadIo) -> bool {
    request
        .filter
        .kinds
        .as_ref()
        .is_some_and(|kinds| kinds.contains(&Kind::EventDeletion))
        && !request.filter.generic_tags.is_empty()
}

pub(crate) fn deletion_failure_executor(io: Arc<DeletionFailureIo>) -> RelayPlanExecutor {
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
