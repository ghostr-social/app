use ghostr_discovery::content::candidates::CandidateRegistry;
use nostr_sdk::{EventBuilder, JsonUtil, Keys, Kind, Tag, Timestamp};

#[test]
fn later_repost_readmits_current_content_for_delivery_priority() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Repost, original.as_json())
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://relay.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .custom_created_at(Timestamp::from(100))
        .sign_with_keys(&reposter)
        .expect("wrapper");
    let mut registry = CandidateRegistry::new();

    assert_eq!(registry.inspect_all(&[original]).admitted.len(), 1);
    let refreshed = registry.inspect_all(&[wrapper]).admitted;

    assert_eq!(refreshed.len(), 1);
    assert_eq!(refreshed[0].post.feed_sort_at, 100);
}

fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("valid tag")
}
