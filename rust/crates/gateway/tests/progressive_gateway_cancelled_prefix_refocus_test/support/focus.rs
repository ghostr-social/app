use crate::gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusAdmission, FocusGeneration, FocusItem};
use std::time::Duration;

const RETAINED_PREVIOUS: usize = 3;

pub async fn focus_and_wait(
    harness: &ProgressiveDeliveryHarness,
    items: &[FocusItem],
    current: usize,
    generation: u64,
) {
    let expected = items[current].post.clone();
    let mut focus = DeliveryFocus::compatibility(items.to_vec(), current, 0);
    focus.generation = FocusGeneration::try_new(generation).expect("generation");
    assert_eq!(
        harness.delivery.handle.update_focus(focus),
        FocusAdmission::Accepted
    );
    let observed = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let accepted = harness.delivery.handle.latest_plan().is_some_and(|plan| {
                plan.focus_generation == Some(generation)
                    && plan.current.as_ref() == Some(&expected)
            });
            if accepted {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    if observed.is_err() {
        panic!(
            "absent causal focus plan: expected generation={generation}, current={expected:?}; \
             demands={:#?}; latest_plan={:#?}",
            harness.delivery.demands(),
            harness.delivery.handle.latest_plan(),
        );
    }
}

pub async fn focus_trimmed_and_wait(
    harness: &ProgressiveDeliveryHarness,
    all: &[FocusItem],
    original: usize,
    generation: u64,
) {
    let first = original.saturating_sub(RETAINED_PREVIOUS);
    focus_and_wait(harness, &all[first..], original - first, generation).await;
}
