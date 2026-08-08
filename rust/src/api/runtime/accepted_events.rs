//! Local admission after a relay accepted a Dart-signed event.

#[cfg(test)]
use crate::api::feed::outcomes::file_lists;
use crate::api::feed::outcomes::file_lists_for;
use crate::api::runtime::discovery::{DiscoveryRuntime, OutcomeSinks};
#[cfg(test)]
use crate::discovery::cache::EventCache;
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::Event;

#[cfg(test)]
pub(crate) async fn remember_accepted(cache: &EventCache, sinks: &OutcomeSinks, event: &Event) {
    cache
        .remember_for(SessionGeneration::initial(), std::slice::from_ref(event))
        .await;
    file_lists(sinks, std::slice::from_ref(event)).await;
}

impl DiscoveryRuntime {
    pub(crate) async fn remember_accepted(&self, session: SessionGeneration, event: &Event) {
        let cache = self.executor.cache();
        let sinks = OutcomeSinks {
            state: self.state.clone(),
            bootstrap: self.bootstrap.clone(),
            candidates: None,
        };
        cache
            .remember_for(session, std::slice::from_ref(event))
            .await;
        file_lists_for(&sinks, session, std::slice::from_ref(event)).await;
    }
}
