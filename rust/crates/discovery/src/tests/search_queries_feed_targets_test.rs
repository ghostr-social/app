//! Plain feed queries route to the outbox relays where the wanted authors
//! actually publish; the mp4 note hunt still needs the search relays and
//! the tag-scoped file query merges both.

use crate::search_queries::{plan_discovery, OutboxLookup, RelayTarget};
use crate::video_filters::DiscoveryRequest;

#[test]
fn plain_feed_targets_outbox_hunt_targets_search() {
    let plan = plan_discovery(&DiscoveryRequest::default());
    let targets: Vec<RelayTarget> = plan
        .queries
        .iter()
        .map(|query| query.target.clone())
        .collect();

    assert_eq!(
        targets,
        vec![
            RelayTarget::OutboxRelays,
            RelayTarget::OutboxRelays,
            RelayTarget::SearchRelays,
            RelayTarget::SearchAndOutboxRelays,
        ]
    );
}

#[test]
fn an_unscoped_feed_uses_the_discovery_relays() {
    let plan = plan_discovery(&DiscoveryRequest::default());

    assert_eq!(plan.outbox, OutboxLookup::DiscoveryRelays);
}
