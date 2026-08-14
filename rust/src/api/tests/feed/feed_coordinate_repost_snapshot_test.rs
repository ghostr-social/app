use crate::api::feed::state::FeedState;
use crate::api::feed_types::FfiFeedRepostTarget;
use crate::api::tests::feed_fixtures::{profile_event, signed_event, SignedEventFixture};
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::{JsonUtil, Keys, Kind};

#[test]
fn coordinate_repost_snapshot_resolves_its_reposter() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::Custom(34_235),
        content: "https://cdn.example/clip.mp4",
        tags: vec![vec!["d".to_owned(), "clip".to_owned()]],
        created_at: 10,
    });
    let coordinate = format!("34235:{}:clip", creator.public_key().to_hex());
    let wrapper_source = original.as_json();
    let wrapper = signed_event(SignedEventFixture {
        keys: &reposter,
        kind: Kind::Custom(16),
        content: &wrapper_source,
        tags: vec![
            vec!["a".to_owned(), coordinate],
            vec!["p".to_owned(), creator.public_key().to_hex()],
            vec!["k".to_owned(), "34235".to_owned()],
        ],
        created_at: 20,
    });
    let mut state = FeedState::new();
    let (feed, open) = state.open(FeedSpec::Following {
        viewer: None,
        follows: vec![reposter.public_key()],
    });

    state.apply(
        &open.expect("following opens").context,
        Ok(vec![
            profile_event(&reposter, r#"{"name":"Relay Bob"}"#, 5),
            wrapper,
        ]),
    );

    let rows = state.snapshot(feed);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.event_id, original.id.to_hex());
    assert_eq!(row.creator.pubkey, creator.public_key().to_hex());
    let repost = row.repost.as_ref().expect("repost metadata");
    assert_eq!(repost.target, FfiFeedRepostTarget::Coordinate);
    assert_eq!(repost.reposter.pubkey, reposter.public_key().to_hex());
    assert_eq!(repost.reposter.display_name, "Relay Bob");
}
