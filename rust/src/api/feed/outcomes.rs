//! Session-safe filing of events returned by discovery work.

use crate::api::runtime::discovery::{lock, OutcomeSinks};
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::Event;

#[cfg(test)]
pub(crate) async fn file_lists(sinks: &OutcomeSinks, events: &[Event]) {
    let session = lock(&sinks.state).session_generation();
    file_lists_for(sinks, session, events).await;
}

pub(crate) async fn file_lists_for(
    sinks: &OutcomeSinks,
    session: SessionGeneration,
    events: &[Event],
) {
    sinks.bootstrap.ingest_for(session, events).await;
    let follows = lock(&sinks.state).ingest_social_for(session, events);
    if let Some(follows) = follows {
        sinks.bootstrap.track_follows_for(session, follows).await;
    }
}
