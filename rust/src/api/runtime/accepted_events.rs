//! Local admission after a relay accepted a Dart-signed event.

use crate::api::feed::outcomes::file_lists_for;
use crate::api::runtime::discovery::{DiscoveryRuntime, OutcomeSinks};

use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::Event;

impl DiscoveryRuntime {
    pub(crate) async fn remember_accepted(&self, session: SessionGeneration, event: &Event) {
        let cache = self.executor.cache();
        let sinks = OutcomeSinks {
            state: std::sync::Arc::clone(&self.state),
            bootstrap: std::sync::Arc::clone(&self.bootstrap),
            candidates: None,
        };
        cache
            .remember_for(session, core::slice::from_ref(event))
            .await;
        file_lists_for(&sinks, session, core::slice::from_ref(event)).await;
    }
}

#[cfg(test)]
#[path = "accepted_events_axiom_test.rs"]
pub(crate) mod axiom_test_support;
