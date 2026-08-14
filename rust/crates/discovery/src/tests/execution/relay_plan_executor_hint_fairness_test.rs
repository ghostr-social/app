use crate::execution::relay_executor::target_enrichment::target_plan;
use crate::query::search::RelayTarget;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use std::collections::BTreeSet;

#[test]
fn every_hint_in_a_normal_protected_repost_page_is_retained() {
    let creator = Keys::generate();
    let wrappers: Vec<_> = (0..9)
        .map(|index| wrapper_with_unique_hint(&creator, index))
        .collect();
    let plan = target_plan(&wrappers).expect("target plan");
    let hints: BTreeSet<_> = plan
        .queries
        .iter()
        .flat_map(|query| match &query.target {
            RelayTarget::HintedRelays(hints) => hints.iter(),
            _ => [].iter(),
        })
        .collect();

    assert_eq!(hints.len(), wrappers.len());
}

fn wrapper_with_unique_hint(creator: &Keys, index: usize) -> Event {
    let original = EventBuilder::new(
        Kind::Custom(21),
        format!("https://cdn.example/v{index}.mp4"),
    )
    .sign_with_keys(creator)
    .expect("original");
    EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&[
                "e",
                &original.id.to_hex(),
                &format!("wss://relay{index}.example"),
            ]),
            tag(&["p", &creator.public_key().to_hex()]),
            tag(&["k", "21"]),
        ])
        .sign_with_keys(&Keys::generate())
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
