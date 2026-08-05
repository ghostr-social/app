//! A successful relay read is merged with the session cache.

use super::relay_pool_owner_support::TestRelayIo;
use crate::discovery::event_cache::EventCache;
use crate::discovery::event_queries::plan_event_queries;
use crate::discovery::relay_fetch::{fetch, RelayFetch};
use crate::discovery::relay_pool_owner::{RelayPoolConfiguration, RelayPoolOwner};
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::{Client, Filter, Kind};
use std::sync::Arc;

#[tokio::test]
async fn successful_empty_answer_remains_successful() {
    let io = Arc::new(TestRelayIo::blocked());
    io.release_query();
    let owner = RelayPoolOwner::with_io(
        Arc::new(Client::default()),
        RelayPoolConfiguration::default(),
        io,
    );
    let session = SessionGeneration::initial();
    let route = owner.begin_route(session).await.expect("current route");
    let query = plan_event_queries(vec![Filter::new().kind(Kind::TextNote)])
        .queries
        .remove(0);

    let events = fetch(RelayFetch {
        route,
        cache: Arc::new(EventCache::session()),
        session,
        relays: Some(vec!["wss://read.example".to_owned()]),
        query,
    })
    .await
    .expect("successful empty relay answer");

    assert!(events.is_empty());
}
