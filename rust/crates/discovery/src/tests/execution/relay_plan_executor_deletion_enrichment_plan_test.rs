use crate::execution::relay_executor::deletion_enrichment::axiom_test_support::deletion_plan;

use crate::tests::support::filter_json;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind, Tag, Timestamp};
use serde_json::json;

#[test]
fn original_deletion_filters_split_event_and_exact_address_targets() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(34235), "https://cdn.example/v.mp4")
        .tags([tag(&["d", " clip "])])
        .custom_created_at(Timestamp::from(10))
        .sign_with_keys(&creator)
        .expect("original");
    let coordinate = format!("34235:{}: clip ", creator.public_key());
    let wrapper = EventBuilder::new(Kind::Custom(16), original.as_json())
        .tags([
            tag(&["e", &original.id.to_hex()]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["a", &coordinate]),
        ])
        .custom_created_at(Timestamp::from(20))
        .sign_with_keys(&reposter)
        .expect("wrapper");

    let plan = deletion_plan(core::slice::from_ref(&wrapper)).expect("deletion lookup");

    assert_eq!(plan.queries.len(), 3);
    let filters: Vec<_> = plan
        .queries
        .iter()
        .map(|query| filter_json(&query.filter))
        .collect();
    assert!(filters.iter().any(|filter| {
        filter["#e"] == json!([original.id.to_hex()])
            && filter["authors"] == json!([creator.public_key().to_hex()])
    }));
    assert!(filters.iter().any(|filter| {
        filter["#e"] == json!([wrapper.id.to_hex()])
            && filter["authors"] == json!([reposter.public_key().to_hex()])
    }));
    let address = filters
        .iter()
        .find(|filter| filter.get("#a").is_some())
        .expect("valid test fixture");
    assert_eq!(address["#a"], json!([coordinate]));
    assert_eq!(address["authors"], json!([creator.public_key().to_hex()]));
    assert!(filters
        .iter()
        .all(|filter| { filter["kinds"] == json!([5]) && filter.get("until").is_none() }));
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
