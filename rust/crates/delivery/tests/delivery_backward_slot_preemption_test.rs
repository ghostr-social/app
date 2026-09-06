//! A reverse swipe must take a constrained origin slot from the outgoing response.
mod delivery_backward_slot_fixture;
mod delivery_fixture;

use core::time::Duration;
use delivery_backward_slot_fixture::outgoing_body;
use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::start_harness;

const TOTAL: u64 = 32;

#[tokio::test]
async fn backward_focus_preempts_an_outgoing_body_that_owns_the_origin_slot() {
    let mut origin = ControlledOrigin::serve(TOTAL).await;
    let mut options = production_geometry_parallel_options();
    options.tuning.max_requests_per_authority = core::num::NonZeroUsize::new(1);
    let harness = start_harness("ghostr-backward-origin-preemption", options);
    let items = vec![
        sized_item("previous", &origin.url_for("previous"), TOTAL, 20_000),
        sized_item("outgoing", &origin.url_for("outgoing"), TOTAL, 20_000),
    ];
    harness
        .handle
        .update_focus(focus_now(vec![items[1].clone()], 0, 0));
    let _outgoing = outgoing_body(&mut origin, &harness).await;

    harness.handle.update_focus(focus_now(items, 0, 0));
    let next = tokio::time::timeout(Duration::from_millis(300), origin.next()).await;
    harness.handle.clear().await.expect("fixture shutdown");
    std::fs::remove_dir_all(&harness.root).expect("fixture cleanup");
    assert_eq!(
        next.expect("new current must not wait for outgoing EOF")
            .path,
        "/previous.mp4"
    );
}
