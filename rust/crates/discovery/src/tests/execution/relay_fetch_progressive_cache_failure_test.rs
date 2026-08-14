use crate::cache::EventCache;
use crate::execution::fetch::{fetch, RelayFetch};
use crate::query::events::plan_event_queries;
use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use crate::tests::event_cache_support::{note, notes};
use nostr_sdk::{Client, Event};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::test]
async fn progressive_cache_fallback_keeps_the_page_retryable() {
    let session = SessionGeneration::initial();
    let cache = Arc::new(EventCache::session());
    let cached = note(100);
    cache
        .remember_for(session, std::slice::from_ref(&cached))
        .await;
    let owner = RelayPoolOwner::with_io(
        Arc::new(Client::default()),
        RelayPoolConfiguration::default(),
        Arc::new(FailingIo),
    );
    let route = owner.begin_route(session).await.expect("route");
    let query = plan_event_queries(vec![notes()]).queries.remove(0);
    let (progress, mut events) = mpsc::channel(1);

    let fetching = tokio::spawn(fetch(RelayFetch {
        route,
        cache,
        session,
        relays: Some(vec!["wss://read.example".to_owned()]),
        query,
        progress: Some(progress),
    }));

    assert_eq!(events.recv().await, Some(cached));
    assert!(fetching.await.expect("task").is_err());
}

struct FailingIo;

impl RelayIo for FailingIo {
    fn read(&self, _: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async { anyhow::bail!("offline") })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}
