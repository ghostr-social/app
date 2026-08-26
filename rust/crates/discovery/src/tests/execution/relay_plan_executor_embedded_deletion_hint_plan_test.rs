use crate::execution::relay_executor::deletion_enrichment::axiom_test_support::deletion_plan;

use crate::query::search::RelayTarget;
use crate::tests::support::filter_json;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind, Tag};
use serde_json::json;

#[test]
fn embedded_repost_original_deletion_reuses_its_event_hint() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::TextNote, "https://cdn.example/v.mp4")
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Repost, original.as_json())
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://target.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
        ])
        .sign_with_keys(&reposter)
        .expect("wrapper");

    let plan = deletion_plan(&[wrapper]).expect("deletion plan");
    let query = plan
        .queries
        .iter()
        .find(|query| filter_json(&query.filter)["#e"] == json!([original.id.to_hex()]));

    assert_eq!(
        query.map(|query| &query.target),
        Some(&RelayTarget::HintedRelays(vec![
            "wss://target.example".to_owned(),
        ])),
    );
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
