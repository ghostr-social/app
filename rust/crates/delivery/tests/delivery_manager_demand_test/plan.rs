use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use ghostr_engine::ByteRange;

const WAIT_LIMIT: Duration = Duration::from_secs(30);
type PlanSummary = (u64, Option<ByteRange>, Vec<ByteRange>);

pub async fn wait_for_demand_plan(
    handle: &DeliveryHandle,
    after: u64,
    post: &str,
    range: ByteRange,
) -> PlanEvidence {
    let result = tokio::time::timeout(WAIT_LIMIT, wait_matching(handle, after, post, range)).await;
    result.unwrap_or_else(|error| {
        panic!(
            "demanded plan publication: {error:?}; plans={:?}",
            plan_summary(handle, post)
        )
    })
}

async fn wait_matching(
    handle: &DeliveryHandle,
    after: u64,
    post: &str,
    range: ByteRange,
) -> PlanEvidence {
    let notifier = handle.plan_notifier();
    loop {
        let changed = notifier.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        if let Some(plan) = matching_plan(handle, after, post, range) {
            return plan;
        }
        changed.await;
    }
}

fn matching_plan(
    handle: &DeliveryHandle,
    after: u64,
    post: &str,
    range: ByteRange,
) -> Option<PlanEvidence> {
    handle
        .plan_history()
        .into_iter()
        .find(|plan| plan.revision > after && first_request(plan, post) == Some(range))
}

fn plan_summary(handle: &DeliveryHandle, post: &str) -> Vec<PlanSummary> {
    handle
        .plan_history()
        .into_iter()
        .map(|plan| summarize_plan(&plan, post))
        .collect()
}

fn summarize_plan(plan: &PlanEvidence, post: &str) -> PlanSummary {
    (
        plan.revision,
        first_request(plan, post),
        retained_requests(plan, post),
    )
}

fn first_request(plan: &PlanEvidence, post: &str) -> Option<ByteRange> {
    plan.plan
        .allocations
        .iter()
        .find(|item| item.post.as_str() == post)
        .map(|item| item.request.requested_bytes())
}

fn retained_requests(plan: &PlanEvidence, post: &str) -> Vec<ByteRange> {
    plan.plan
        .retained
        .iter()
        .filter(|item| item.post.as_str() == post)
        .map(|item| item.request.requested_bytes())
        .collect()
}
