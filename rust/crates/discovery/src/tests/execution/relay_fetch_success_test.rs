//! A successful relay read is merged with the session cache.

use crate::cache::EventCache;
use crate::execution::fetch::{fetch, RelayFetch};
use crate::query::events::plan_event_queries;
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use crate::test_support::TestRelayIo;
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
        progress: None,
    })
    .await
    .expect("successful empty relay answer");

    assert!(events.is_empty());
}
