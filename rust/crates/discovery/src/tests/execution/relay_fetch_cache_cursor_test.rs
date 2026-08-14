use crate::cache::EventCache;
use crate::execution::collector::collect_page;
use crate::execution::fetch::{fetch, RelayFetch};
use crate::query::events::plan_event_queries;
use crate::query::search::QueryRole;
use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo};
use crate::relay::pool::{RelayPoolConfiguration, RelayPoolOwner};
use crate::session_generation::SessionGeneration;
use nostr_sdk::{Client, Event, EventBuilder, Filter, Keys, Kind, Timestamp};
use std::sync::Arc;

#[tokio::test]
async fn cached_history_does_not_move_the_fresh_page_cursor() {
    let session = SessionGeneration::initial();
    let cache = Arc::new(EventCache::session());
    cache.remember_for(session, &[note(100)]).await;
    let io = Arc::new(ReturningIo(vec![note(300), note(200)]));
    let owner = RelayPoolOwner::with_io(
        Arc::new(Client::default()),
        RelayPoolConfiguration::default(),
        io,
    );
    let query = plan_event_queries(vec![Filter::new().kind(Kind::TextNote).limit(2)])
        .queries
        .remove(0);
    let route = owner.begin_route(session).await.expect("route");

    let page = collect_page(vec![(
        QueryRole::Primary,
        tokio::spawn(fetch(RelayFetch {
            route,
            cache,
            session,
            relays: Some(vec!["wss://read.example".to_owned()]),
            query,
            progress: None,
        })),
    )])
    .await
    .expect("page");

    assert_eq!(page.events.len(), 3);
    assert_eq!(page.cursor, Some(Timestamp::from(199)));
}

fn note(created_at: u64) -> Event {
    EventBuilder::text_note(format!("note {created_at}"))
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("note")
}

struct ReturningIo(Vec<Event>);

impl RelayIo for ReturningIo {
    fn read(&self, _: RelayReadIo) -> RelayIoFuture<'_, Vec<Event>> {
        Box::pin(async { Ok(self.0.clone()) })
    }

    fn broadcast(&self, _: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}
