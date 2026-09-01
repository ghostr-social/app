use super::PREFIX;
use ghostr_delivery::delivery_events::PlanEvidence;

pub(super) fn assert_prefix_allocation(plan: &PlanEvidence) {
    let allocations: Vec<_> = plan
        .plan
        .allocations
        .iter()
        .map(|item| {
            (
                item.post.as_str(),
                item.request.requested_bytes(),
                item.authority,
                item.reason,
            )
        })
        .collect();
    let retained: Vec<_> = plan
        .plan
        .retained
        .iter()
        .map(|item| {
            (
                item.action_id,
                item.post.as_str(),
                item.request.requested_bytes(),
                item.reason,
            )
        })
        .collect();
    assert!(
        plan.plan.allocations.iter().any(|item| {
            let bytes = item.request.requested_bytes();
            item.post.as_str() == "p6" && bytes.start == PREFIX.start && bytes.end >= PREFIX.end
        }),
        "live demand {PREFIX:?} missing after refocus; allocations={allocations:?}; retained={retained:?}"
    );
}
