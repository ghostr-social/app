use crate::cache::client_with_event_cache;
use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::execution::relay_executor::RelayPlanExecutor;
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::tests::outbox_support::{empty_directory, BOOTSTRAP_RELAY};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::{Event, Filter, Kind};
use std::sync::{Arc, Mutex};

pub(crate) struct ProfileIo {
    primary_kind: Kind,
    primary: Event,
    return_primary: bool,
    profile: Event,
    pub(crate) filters: Mutex<Vec<Filter>>,
}

impl ProfileIo {
    pub(crate) fn new(primary_kind: Kind, primary: Event, profile: Event) -> Arc<Self> {
        Arc::new(Self {
            primary_kind,
            primary,
            return_primary: true,
            profile,
            filters: Mutex::new(Vec::new()),
        })
    }

    pub(crate) fn empty(primary_kind: Kind, placeholder: Event) -> Arc<Self> {
        Arc::new(Self {
            primary_kind,
            primary: placeholder.clone(),
            profile: placeholder,
            filters: Mutex::new(Vec::new()),
            return_primary: false,
        })
    }
}

impl RelayIo for ProfileIo {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async move {
            self.filters
                .lock()
                .expect("filters")
                .push(request.filter.clone());
            if has_kind(&request.filter, Kind::Metadata) {
                return Ok(vec![self.profile.clone()]);
            }
            if self.answers_primary(&request.filter) {
                return Ok(vec![self.primary.clone()]);
            }
            Ok(Vec::new())
        })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

impl ProfileIo {
    fn answers_primary(&self, filter: &Filter) -> bool {
        self.return_primary
            && has_kind(filter, self.primary_kind)
            && (self.primary_kind != Kind::TextNote || filter.search.is_none())
    }
}

pub(crate) fn executor(io: Arc<ProfileIo>) -> RelayPlanExecutor {
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
