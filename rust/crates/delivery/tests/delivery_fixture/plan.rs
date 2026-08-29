//! Event-driven waits for published delivery plans.

use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use ghostr_engine::{ByteRange, PostId};
use tokio::time::{timeout_at, Instant};

const PLAN_WAIT_LIMIT: Duration = Duration::from_secs(30);
const DIAGNOSTIC_PLAN_LIMIT: usize = 8;
type PlanSummary = (u64, Option<PostId>, Vec<(PostId, ByteRange)>);

pub async fn wait_for_current(handle: &DeliveryHandle, expected: &str) {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if handle
                .latest_plan()
                .and_then(|plan| plan.current)
                .is_some_and(|post| post.as_str() == expected)
            {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("focused plan publication");
}

pub async fn wait_for_plan(
    handle: &DeliveryHandle,
    after_revision: u64,
    ready: impl Fn(&PlanEvidence) -> bool,
) -> PlanEvidence {
    let notifier = handle.plan_notifier();
    let deadline = Instant::now() + PLAN_WAIT_LIMIT;
    loop {
        let changed = notifier.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        if let Some(plan) = matching(handle, after_revision, &ready) {
            return plan;
        }
        if timeout_at(deadline, changed).await.is_err() {
            return boundary(handle, after_revision, &ready);
        }
    }
}

fn boundary(
    handle: &DeliveryHandle,
    after_revision: u64,
    ready: &impl Fn(&PlanEvidence) -> bool,
) -> PlanEvidence {
    matching(handle, after_revision, ready).unwrap_or_else(|| panic_missing(handle, after_revision))
}

fn matching(
    handle: &DeliveryHandle,
    after_revision: u64,
    ready: &impl Fn(&PlanEvidence) -> bool,
) -> Option<PlanEvidence> {
    handle
        .plan_history()
        .into_iter()
        .find(|plan| plan.revision > after_revision && ready(plan))
}

fn panic_missing(handle: &DeliveryHandle, after_revision: u64) -> ! {
    let history = handle.plan_history();
    let bounds = history
        .first()
        .zip(history.last())
        .map(|(first, last)| (first.revision, last.revision));
    panic!(
        "matching plan after revision {after_revision}; retained_bounds={bounds:?}; recent={:?}",
        summaries(&history)
    )
}

fn summaries(history: &[PlanEvidence]) -> Vec<PlanSummary> {
    history
        .iter()
        .rev()
        .take(DIAGNOSTIC_PLAN_LIMIT)
        .map(|plan| {
            let evictions = plan
                .plan
                .evictions
                .iter()
                .take(DIAGNOSTIC_PLAN_LIMIT)
                .map(|eviction| (eviction.post.clone(), eviction.range))
                .collect();
            (plan.revision, plan.current.clone(), evictions)
        })
        .collect()
}
