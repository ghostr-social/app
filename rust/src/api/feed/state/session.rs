//! Account-session transitions for [`FeedState`].

use super::FeedState;
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::content::profiles::ProfileStore;
use crate::discovery::session_generation::SessionGeneration;
use crate::discovery::content::social_graph::SocialGraph;
use nostr_sdk::{Event, Keys, PublicKey};

impl FeedState {
    pub(crate) fn session_generation(&self) -> SessionGeneration {
        self.session
    }

    #[cfg(test)]
    pub(crate) fn ingest_social(&mut self, events: &[Event]) -> Option<Vec<PublicKey>> {
        self.ingest_social_for(self.session, events)
    }

    pub(crate) fn ingest_social_for(
        &mut self,
        session: SessionGeneration,
        events: &[Event],
    ) -> Option<Vec<PublicKey>> {
        if self.session != session {
            return None;
        }
        self.graph
            .ingest_all(events)
            .then(|| self.graph.follow_list())
    }

    /// Drops account models, closes streams, and advances async context.
    pub(crate) fn reset_session(&mut self) -> SessionGeneration {
        self.session = self.session.next();
        self.store.reset_session();
        self.candidates.clear();
        self.profiles = ProfileStore::new();
        self.graph = SocialGraph::new(Keys::generate().public_key());
        self.feeds.clear();
        self.session
    }

    /// Re-adopting one viewer preserves lists already ingested for them.
    pub(super) fn adopt_viewer(&mut self, spec: &FeedSpec) {
        if let FeedSpec::MainFeed {
            viewer: Some(viewer),
        } = spec
        {
            if !self.graph.belongs_to(viewer) {
                self.graph = SocialGraph::new(*viewer);
            }
        }
    }
}
