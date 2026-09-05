use super::delivery_fixture::plan::wait_for_plan;
use super::delivery_fixture::DeliveryHarness;
use super::focus_fixture::generated_focus;
use ghostr_delivery::delivery_events::{FocusAdmission, FocusItem, PlanEvidence};

const RETAINED_PREVIOUS: usize = 3;
const FIRST_FORWARD_GENERATION: u64 = 8;

pub(super) async fn sequential_refocus(
    harness: &DeliveryHarness,
    all: &[FocusItem],
) -> PlanEvidence {
    let mut latest = None;
    for original_index in 1..all.len() {
        let offset = u64::try_from(original_index - 1).expect("bounded fixture index");
        let generation = FIRST_FORWARD_GENERATION + offset;
        latest = Some(focus(harness, all, original_index, generation).await);
    }
    latest.expect("fixture has a target after the initial post")
}

async fn focus(
    harness: &DeliveryHarness,
    all: &[FocusItem],
    original_index: usize,
    generation: u64,
) -> PlanEvidence {
    let first = original_index.saturating_sub(RETAINED_PREVIOUS);
    let visible = all[first..].to_vec();
    let current = original_index - first;
    let expected = all[original_index].post.clone();
    let after = harness.handle.latest_plan().map_or(0, |plan| plan.revision);
    let update = generated_focus(visible, current, generation);
    assert_eq!(
        harness.handle.update_focus(update),
        FocusAdmission::Accepted,
        "refocus generation is accepted"
    );
    wait_for_plan(&harness.handle, after, |plan| {
        plan.focus_generation == Some(generation) && plan.current.as_ref() == Some(&expected)
    })
    .await
}
