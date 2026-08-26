use crate::execution::relay_executor::target_enrichment::axiom_test_support::target_plan;

use crate::query::search::RelayTarget;
use crate::tests::support::filter_json;
use nostr_sdk::{EventBuilder, JsonUtil as _, Keys, Kind, Tag};
use serde_json::json;

#[test]
fn coordinate_repost_queries_the_exact_current_address_without_a_cutoff() {
    let creator = Keys::generate();
    let original = EventBuilder::new(Kind::Custom(34235), "https://cdn.example/v.mp4")
        .tags([tag(&["d", " clip "])])
        .sign_with_keys(&creator)
        .expect("original");
    let coordinate = format!("34235:{}: clip ", creator.public_key());
    let wrapper = EventBuilder::new(Kind::Custom(16), original.as_json())
        .tags([tag(&["a", &coordinate, "wss://coordinate.example"])])
        .sign_with_keys(&Keys::generate())
        .expect("wrapper");

    let plan = target_plan(core::slice::from_ref(&wrapper)).expect("target lookup");
    assert_eq!(plan.queries.len(), 1);
    let filter = filter_json(&plan.queries[0].filter);

    assert_eq!(filter["authors"], json!([creator.public_key().to_hex()]));
    assert_eq!(filter["kinds"], json!([34235]));
    assert_eq!(filter["#d"], json!([" clip "]));
    assert!(filter.get("until").is_none());
    assert!(matches!(
        &plan.queries[0].target,
        RelayTarget::HintedRelays(hints) if hints == &["wss://coordinate.example"]
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
