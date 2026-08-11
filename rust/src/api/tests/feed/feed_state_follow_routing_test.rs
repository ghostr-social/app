//! The main feed's NIP-65 routing over the feed state: a cold open
//! dispatches its first page immediately, routed by nothing but the
//! bootstrap relays, and the viewer's kind-3 landing afterwards routes
//! every later request by the follows — without ever filtering by them.

use crate::api::feed::state::FeedState;
use crate::api::runtime::discovery::{lock, SharedFeedState};
use crate::api::tests::feed_fixtures::{signed_event, SignedEventFixture};
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::{Event, Keys, Kind, PublicKey};
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

fn state() -> SharedFeedState {
    Arc::new(Mutex::new(FeedState::new()))
}

/// The first page never waits for a relay list: it is dispatched while
/// the graph is still cold, so it queries the bootstrap relays.
#[tokio::test]
async fn a_cold_open_dispatches_its_first_page_unrouted() {
    let viewer = Keys::generate();
    let state = state();

    let (_, dispatch) = lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });

    let open = dispatch.expect("the main feed always dispatches a first page");
    assert!(open.request.routing_authors.is_empty());
}

#[tokio::test]
async fn a_landed_follow_list_routes_the_next_request() {
    let viewer = Keys::generate();
    let follow = Keys::generate().public_key();
    let state = state();
    lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });

    let follows = lock(&state).ingest_social(&[contact_list(&viewer, &follow)]);

    assert_eq!(follows, Some(vec![follow]));
    let (_, dispatch) = lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });
    let request = dispatch.expect("the main feed always dispatches").request;
    assert_eq!(request.routing_authors, vec![follow]);
    assert!(request.authors.is_empty(), "routing must not filter");
}

/// Someone else's follow list is not the viewer's routing set.
#[tokio::test]
async fn another_accounts_follow_list_is_ignored() {
    let viewer = Keys::generate();
    let stranger = Keys::generate();
    let state = state();
    lock(&state).open(FeedSpec::MainFeed {
        viewer: Some(viewer.public_key()),
    });

    let follows = lock(&state).ingest_social(&[contact_list(&stranger, &stranger.public_key())]);

    assert_eq!(follows, None);
}
