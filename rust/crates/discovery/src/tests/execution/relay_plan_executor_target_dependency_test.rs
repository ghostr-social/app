use crate::execution::relay_executor::target_enrichment::axiom_test_support::target_plan_with_dependencies;
use nostr_sdk::{Event, EventBuilder, EventId, Keys, Kind, PublicKey, Tag};
use std::collections::BTreeMap;

#[test]
fn same_event_id_keeps_each_author_and_kind_constraint_attached() {
    let target = EventId::all_zeros();
    let first_author = Keys::generate().public_key();
    let second_author = Keys::generate().public_key();
    let first = wrapper(target, first_author, 21);
    let second = wrapper(target, second_author, 21);
    let third = wrapper(target, first_author, 22);
    let expected = BTreeMap::from([
        ((first_author.to_hex(), 21), first.id),
        ((second_author.to_hex(), 21), second.id),
        ((first_author.to_hex(), 22), third.id),
    ]);

    let (plan, dependencies, unplanned) =
        target_plan_with_dependencies(&[first, second, third]).expect("target plan");

    assert_eq!(plan.queries.len(), 3);
    assert!(unplanned.is_empty());
    for (query, dependency) in plan.queries.iter().zip(dependencies) {
        let author = query
            .filter
            .authors
            .as_ref()
            .and_then(|authors| authors.first())
            .expect("author")
            .to_hex();
        let kind = query
            .filter
            .kinds
            .as_ref()
            .and_then(|kinds| kinds.first())
            .expect("kind")
            .as_u16();
        assert_eq!(
            dependency.into_iter().collect::<Vec<_>>(),
            [expected[&(author, kind)]]
        );
    }
}

fn wrapper(target: EventId, author: PublicKey, kind: u16) -> Event {
    EventBuilder::new(Kind::Custom(16), "")
        .tags([
            tag(&["e", &target.to_hex()]),
            tag(&["p", &author.to_hex()]),
            tag(&["k", &kind.to_string()]),
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
    .expect("valid tag")
}
