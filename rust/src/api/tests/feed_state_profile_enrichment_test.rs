//! Kind-0 events riding in a retrieval outcome enrich the creator
//! identity of every snapshot row (profile_store precedence rules).

use crate::api::feed_state::FeedState;
use crate::api::tests::feed_fixtures::{profile_event, video_note};
use crate::discovery::feed_spec::FeedSpec;
use nostr_sdk::Keys;

#[test]
fn metadata_in_the_outcome_names_the_creator() {
    let mut state = FeedState::new();
    let keys = Keys::generate();
    let (feed, dispatch) = state.open(FeedSpec::MainFeed { viewer: keys.public_key() });
    let open = dispatch.expect("main feeds dispatch a first page");

    let metadata = profile_event(&keys, r#"{"name":"Vera","picture":"https://cdn.example/a.png"}"#, 5);
    state.apply(&open.context, Ok(vec![metadata, video_note(&keys, "clip", 40)]));

    let posts = state.snapshot(feed);
    assert_eq!(posts.len(), 1, "metadata events are not feed rows");
    assert_eq!(posts[0].creator.display_name, "Vera");
    assert_eq!(posts[0].creator.avatar_url.as_deref(), Some("https://cdn.example/a.png"));
}
