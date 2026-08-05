//! Local admission after a relay accepted a Dart-signed event.

#[cfg(test)]
use crate::api::feed_outcomes::file_lists;
use crate::api::feed_outcomes::file_lists_for;
use crate::api::feed_runtime::{DiscoveryRuntime, OutcomeSinks};
#[cfg(test)]
use crate::discovery::event_cache::EventCache;
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::Event;

#[cfg(test)]
pub(crate) async fn remember_accepted(cache: &EventCache, sinks: &OutcomeSinks, event: &Event) {
    cache.remember(std::slice::from_ref(event)).await;
    file_lists(sinks, std::slice::from_ref(event)).await;
}

impl DiscoveryRuntime {
    pub(crate) async fn remember_accepted(&self, session: SessionGeneration, event: &Event) {
        let cache = self.executor.cache();
        let sinks = OutcomeSinks {
            state: self.state.clone(),
            bootstrap: self.bootstrap.clone(),
        };
        cache
            .remember_for(session, std::slice::from_ref(event))
            .await;
        file_lists_for(&sinks, session, std::slice::from_ref(event)).await;
    }
}
