//! An accepted local write is immediately queryable and updates feed routing.

use crate::api::feed::state::FeedState;
use crate::api::runtime::accepted_events::axiom_test_support::remember_accepted;
use crate::api::runtime::discovery::{lock, OutcomeSinks, SharedFeedState};
use crate::api::tests::feed_fixtures::{signed_event, SignedEventFixture};
use crate::api::tests::outbox_runtime_support::test_bootstrap;
use crate::discovery::cache::{session_event_database, EventCache};
use crate::discovery::feed::spec::FeedSpec;
use crate::discovery::session_generation::SessionGeneration;
use nostr_sdk::{Event, Filter, Keys, Kind, PublicKey};
use std::sync::{Arc, Mutex};

fn contact_list(viewer: &Keys, follow: &PublicKey) -> Event {
    let tags = vec![vec!["p".to_owned(), follow.to_hex()]];
    signed_event(SignedEventFixture {
        keys: viewer,
        kind: Kind::ContactList,
        content: "",
        tags,
        created_at: 20,
    })
}

#[tokio::test]
async fn accepted_contact_list_is_cached_and_routes_the_next_feed() {
    let viewer = Keys::generate();
    let follow = Keys::generate().public_key();
    let state: SharedFeedState = Arc::new(Mutex::new(FeedState::new()));
    lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    let (bootstrap, _probe) = test_bootstrap();
    let sinks = OutcomeSinks {
        state: std::sync::Arc::clone(&state),
        bootstrap,
        candidates: None,
    };
    let cache = EventCache::new(Arc::new(session_event_database(16)));

    remember_accepted(&cache, &sinks, &contact_list(&viewer, &follow)).await;

    let stored = cache
        .stored_for(
            SessionGeneration::initial(),
            &Filter::new().kind(Kind::ContactList),
        )
        .await
        .expect("current session");
    assert_eq!(stored.len(), 1);
    let (_, dispatch) = lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    assert_eq!(
        dispatch.expect("main feed").request.routing_authors,
        vec![follow]
    );
}
