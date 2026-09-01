use super::delivery_fixture::concurrency_origin::ControlledOrigin;
use super::delivery_fixture::items::{focus_now, seed_range, sized_item};
use super::delivery_fixture::plan::wait_for_plan;
use super::delivery_fixture::DeliveryHarness;
use super::TOTAL;
use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusAdmission, FocusGeneration, FocusItem};

pub(super) async fn seed_prefix(harness: &DeliveryHarness, items: &[FocusItem]) {
    for item in &items[..6] {
        seed_range(&harness.store, item, 0, &[7; TOTAL as usize]).await;
    }
}

pub(super) async fn focus_and_wait(
    harness: &DeliveryHarness,
    items: &[FocusItem],
    current: usize,
    generation: u64,
) {
    let after = harness.handle.latest_plan().map_or(0, |plan| plan.revision);
    let focus = generated_focus(items.to_vec(), current, generation);
    assert_eq!(harness.handle.update_focus(focus), FocusAdmission::Accepted);
    let plan = wait_for_plan(&harness.handle, after, |plan| {
        plan.focus_generation == Some(generation)
    })
    .await;
    assert_eq!(
        plan.current.as_ref().map(|post| post.as_str()),
        Some(ids()[current])
    );
}

pub(super) async fn wait_response_open(harness: &DeliveryHarness, key: &str) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !harness.store.response_open_for_test(key).await {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("headers are authorized before the first body byte");
}

pub(super) fn generated_focus(
    items: Vec<FocusItem>,
    current: usize,
    generation: u64,
) -> DeliveryFocus {
    let mut focus = focus_now(items, current, 0);
    focus.generation = FocusGeneration::try_new(generation).expect("positive focus generation");
    focus
}

pub(super) fn roster(origin: &ControlledOrigin) -> Vec<FocusItem> {
    ids()
        .into_iter()
        .map(|id| sized_item(id, &origin.url_for(id), TOTAL, 4_000))
        .collect()
}

pub(super) fn ids() -> [&'static str; 7] {
    ["p0", "p1", "p2", "p3", "p4", "p5", "p6"]
}
