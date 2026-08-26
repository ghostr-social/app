use crate::execution::relay_executor::deletion_enrichment::axiom_test_support::deletion_plan;

use crate::query::search::RelayTarget;
use crate::tests::support::filter_json;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use serde_json::json;

#[test]
fn original_deletion_lookup_reuses_its_verified_repost_hint() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .tags([tag(&["-"])])
        .sign_with_keys(&creator)
        .expect("original");
    let wrapper = EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&["e", &original.id.to_hex(), "wss://target.example"]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "21"]),
        ])
        .sign_with_keys(&reposter)
        .expect("wrapper");

    let plan = deletion_plan(&[wrapper.clone(), original.clone()]).expect("deletion plan");
    let original_query = plan
        .queries
        .iter()
        .find(|query| filter_json(&query.filter)["#e"] == json!([original.id.to_hex()]));
    let wrapper_query = plan
        .queries
        .iter()
        .find(|query| filter_json(&query.filter)["#e"] == json!([wrapper.id.to_hex()]));

    assert_eq!(
        original_query.map(|query| &query.target),
        Some(&RelayTarget::HintedRelays(vec![
            "wss://target.example".to_owned(),
        ])),
    );
    assert_eq!(
        wrapper_query.map(|query| &query.target),
        Some(&RelayTarget::SearchAndOutboxRelays),
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
