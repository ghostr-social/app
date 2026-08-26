use crate::cache::EventCache;
use crate::execution::fetch::{fetch, RelayFetch};
use crate::query::events::plan_event_queries;
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use crate::test_support::TestRelayIo;
use crate::tests::scheduler_support::note_at;
use nostr_sdk::{Client, Filter, Kind};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn cached_matches_arrive_before_the_network_finishes() {
    let io = Arc::new(TestRelayIo::blocked());
    let owner = RelayPoolOwner::with_io(
        Arc::new(Client::default()),
        RelayPoolConfiguration::default(),
        std::sync::Arc::<TestRelayIo>::clone(&io),
    );
    let session = SessionGeneration::initial();
    let route = owner.begin_route(session).await.expect("current route");
    let filter = Filter::new().kind(Kind::TextNote);
    let query = plan_event_queries(vec![filter]).queries.remove(0);
    let event = note_at(40);
    let cache = Arc::new(EventCache::session());
    cache
        .remember_for(session, core::slice::from_ref(&event))
        .await;
    let (progress, mut updates) = mpsc::channel(1);

    let fetching = tokio::spawn(fetch(RelayFetch {
        route,
        cache,
        session,
        relays: Some(vec!["wss://read.example".to_owned()]),
        query,
        progress: Some(progress),
    }));

    assert_eq!(updates.recv().await, Some(event.clone()));
    assert!(!fetching.is_finished());
    io.release_query();
    let fetched = fetching.await.expect("fetch task").expect("fetch");
    assert_eq!(fetched.events, vec![event]);
    assert_eq!(fetched.fresh_boundary, None);
}
