//! Scheduler-only transformations of discovery plans.

use crate::discovery::search_queries::{plan_discovery, QueryPlan};
use crate::discovery::video_filters::{DiscoveryRequest, WIDE_QUERY_LIMIT};

/// Reissues the primary video query at the wide hunger limit.
pub(crate) fn widened_plan(request: &DiscoveryRequest) -> QueryPlan {
    let mut plan = plan_discovery(request);
    if let Some(primary) = plan.queries.first_mut() {
        primary.filter = primary.filter.clone().limit(WIDE_QUERY_LIMIT);
    }
    plan
}
