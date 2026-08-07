//! The dedicated video query is primary and note/file queries are additive;
//! every planned role still participates in settling the page boundary.

use crate::search_queries::{plan_discovery, QueryRole};
use crate::video_filters::DiscoveryRequest;

#[test]
fn the_first_query_is_primary_and_the_rest_are_additive() {
    let plan = plan_discovery(&DiscoveryRequest::default());
    let roles: Vec<QueryRole> = plan
        .queries
        .iter()
        .map(|query| query.role.clone())
        .collect();

    assert_eq!(
        roles,
        vec![
            QueryRole::Primary,
            QueryRole::Additive,
            QueryRole::Additive,
            QueryRole::Additive,
        ]
    );
}

#[test]
fn search_plans_keep_the_video_query_primary() {
    let plan = plan_discovery(&DiscoveryRequest {
        search_query: Some("skate".into()),
        ..DiscoveryRequest::default()
    });

    assert_eq!(plan.queries[0].role, QueryRole::Primary);
    assert!(plan.queries[1..]
        .iter()
        .all(|query| query.role == QueryRole::Additive));
}
