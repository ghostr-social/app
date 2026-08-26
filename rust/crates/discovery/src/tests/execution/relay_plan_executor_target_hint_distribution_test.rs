use crate::execution::relay_executor::target_enrichment::axiom_test_support::target_plan;

use crate::query::search::RelayTarget;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use std::collections::BTreeSet;

#[test]
fn target_filter_keeps_one_hint_per_target_before_bounded_fallbacks() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let wrappers: Vec<_> = (0..3)
        .flat_map(|target| wrappers_for_target(&creator, &reposter, target))
        .collect();

    let plan = target_plan(&wrappers).expect("target plan");
    assert_eq!(plan.queries.len(), 1);
    let RelayTarget::HintedRelays(hints) = &plan.queries[0].target else {
        panic!("hinted route");
    };
    let selected: BTreeSet<_> = hints.iter().map(String::as_str).collect();

    assert_eq!(selected.len(), 8);
    for target in 0..3 {
        assert!(selected.contains(format!("wss://t{target}h0.example").as_str()));
    }
}

fn wrappers_for_target(creator: &Keys, reposter: &Keys, target: usize) -> Vec<Event> {
    let original = EventBuilder::new(
        Kind::Custom(21),
        format!("https://cdn.example/video{target}.mp4"),
    )
    .sign_with_keys(creator)
    .expect("original");
    (0..5)
        .map(|hint| wrapper(reposter, &original, target, hint))
        .collect()
}

fn wrapper(reposter: &Keys, original: &Event, target: usize, hint: usize) -> Event {
    EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&[
                "e",
                &original.id.to_hex(),
                &format!("wss://t{target}h{hint}.example"),
            ]),
            tag(&["p", &original.pubkey.to_hex()]),
            tag(&["k", "21"]),
        ])
        .sign_with_keys(reposter)
        .expect("wrapper")
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
