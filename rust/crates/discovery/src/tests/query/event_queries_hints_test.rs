use crate::query::events::{plan_hinted_event_queries, HintedEventFilter};
use crate::query::search::RelayTarget;
use nostr_sdk::{Alphabet, Filter, SingleLetterTag};

#[test]
fn hinted_event_planning_keeps_each_route_attached_to_its_filter() {
    let first = HintedEventFilter::new(tagged("first"), vec!["wss://one.example".to_owned()]);
    let second = HintedEventFilter::new(tagged("second"), vec!["wss://two.example".to_owned()]);

    let plan = plan_hinted_event_queries(vec![first, second]);

    assert_eq!(
        plan.queries[0].target,
        RelayTarget::HintedRelays(vec!["wss://one.example".to_owned()]),
    );
    assert_eq!(
        plan.queries[1].target,
        RelayTarget::HintedRelays(vec!["wss://two.example".to_owned()]),
    );
}

fn tagged(value: &str) -> Filter {
    Filter::new().custom_tag(SingleLetterTag::lowercase(Alphabet::E), [value])
}
