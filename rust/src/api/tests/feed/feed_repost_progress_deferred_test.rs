use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedStage;
use crate::api::tests::feed_fixtures::video_note;
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind, Tag};

#[test]
fn repost_progress_waits_for_settled_deletion_checks() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = video_note(&creator, "original", 20);
    let wrapper = EventBuilder::new(Kind::Repost, original.as_json())
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://relay.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::Search("original".to_owned()));
    let context = open.expect("search dispatch").context;

    let candidate = state.apply_progress(&context, &wrapper);

    assert!(candidate.is_none());
    assert_eq!(state.stage(feed), FfiFeedStage::Loading);
    assert!(state.snapshot(feed).is_empty());
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("tag")
}
