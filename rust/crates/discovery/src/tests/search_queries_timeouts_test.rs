//! Search relays keep answering after the fast ones went quiet: discovery
//! queries (term or tag filter) wait 8 seconds, canonical feed queries 5
//! to preserve the distinct latency contracts.

use std::time::Duration;

use crate::search_queries::{
    plan_discovery, DISCOVERY_QUERY_TIMEOUT, FEED_QUERY_TIMEOUT,
};
use crate::video_filters::DiscoveryRequest;

#[test]
fn timeout_constants_match_the_query_contract() {
    assert_eq!(FEED_QUERY_TIMEOUT, Duration::from_secs(5));
    assert_eq!(DISCOVERY_QUERY_TIMEOUT, Duration::from_secs(8));
}

#[test]
fn plain_feed_queries_wait_five_seconds_hunts_wait_eight() {
    let plan = plan_discovery(&DiscoveryRequest::default());
    let timeouts: Vec<Duration> = plan.queries.iter().map(|query| query.timeout).collect();

    assert_eq!(
        timeouts,
        vec![
            FEED_QUERY_TIMEOUT,
            FEED_QUERY_TIMEOUT,
            DISCOVERY_QUERY_TIMEOUT,
            DISCOVERY_QUERY_TIMEOUT,
        ]
    );
}

#[test]
fn tag_filtered_plans_are_discovery_throughout() {
    let plan = plan_discovery(&DiscoveryRequest {
        hashtags: vec!["surf".into()],
        ..DiscoveryRequest::default()
    });

    assert!(plan
        .queries
        .iter()
        .all(|query| query.timeout == DISCOVERY_QUERY_TIMEOUT));
}
