use crate::execution::relay_executor::target_enrichment::target_plan;
use crate::query::search::RelayTarget;
use crate::tests::support::filter_json;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use serde_json::json;

#[test]
fn empty_repost_queries_its_exact_original_without_a_page_cutoff() {
    let creator = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Repost, "")
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://relay.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("wrapper");

    let plan = target_plan(std::slice::from_ref(&wrapper)).expect("target lookup");
    assert_eq!(plan.queries.len(), 1);
    let filter = filter_json(&plan.queries[0].filter);

    assert_eq!(filter["ids"], json!([original.id.to_hex()]));
    assert_eq!(filter["authors"], json!([creator.public_key().to_hex()]));
    assert!(filter.get("until").is_none());
    assert!(matches!(
        &plan.queries[0].target,
        RelayTarget::HintedRelays(relays) if relays == &["wss://relay.example"]
    ));
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
