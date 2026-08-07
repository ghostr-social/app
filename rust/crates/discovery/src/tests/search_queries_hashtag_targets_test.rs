//! Tag-filtered queries hit the search relays' deep tag indexes merged
//! with the outbox; the term-carrying mp4 hunt stays on the search relays
//! alone.

use crate::search_queries::{plan_discovery, RelayTarget};
use crate::video_filters::DiscoveryRequest;

#[test]
fn hashtag_queries_merge_search_and_outbox_relays() {
    let plan = plan_discovery(&DiscoveryRequest {
        hashtags: vec!["surf".into()],
        ..DiscoveryRequest::default()
    });
    let targets: Vec<RelayTarget> = plan
        .queries
        .iter()
        .map(|query| query.target.clone())
        .collect();

    assert_eq!(
        targets,
        vec![
            RelayTarget::SearchAndOutboxRelays,
            RelayTarget::SearchAndOutboxRelays,
            RelayTarget::SearchRelays,
            RelayTarget::SearchAndOutboxRelays,
        ]
    );
}
