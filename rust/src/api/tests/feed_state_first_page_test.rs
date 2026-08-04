//! Opening a feed dispatches its first page and a landed page becomes
//! the visible snapshot: newest first (feed_assembly parity), with the
//! shortened-npub fallback identity from the profile store.

use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed_spec::FeedSpec;
use crate::discovery::video_filters::DiscoveryRequest;
use nostr_sdk::Keys;

#[test]
fn the_first_landed_page_becomes_the_snapshot() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, dispatch) = state.open(FeedSpec::MainFeed { viewer: Some(keys.public_key()) });
    let open = dispatch.expect("main feeds dispatch a first page");
    assert_eq!(open.request, DiscoveryRequest::default());

    let revisions = state.subscribe(feed).expect("open feeds subscribe");
    let events = vec![video_note(&keys, "older", 30), video_note(&keys, "newer", 40)];
    state.apply(&open.context, Ok(events));

    assert!(revisions.has_changed().expect("revision sender alive"));
    let posts = state.snapshot(feed);
    assert_eq!(posts.len(), 2);
    assert_eq!((posts[0].created_at, posts[1].created_at), (40, 30));
    assert!(posts[0].creator.display_name.starts_with("npub1"));
    assert!(posts[0].creator.display_name.ends_with('…'));
    assert!(posts[0].creator.handle.starts_with("@npub1"));
}

#[test]
fn outcomes_for_unknown_contexts_are_ignored() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, dispatch) = state.open(FeedSpec::MainFeed { viewer: Some(keys.public_key()) });
    let open = dispatch.expect("main feeds dispatch a first page");
    state.close(feed);
    state.apply(&open.context, Ok(vec![video_note(&keys, "late", 50)]));
    assert!(state.snapshot(feed).is_empty());
}
