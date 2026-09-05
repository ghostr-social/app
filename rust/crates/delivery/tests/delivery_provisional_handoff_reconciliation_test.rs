mod delivery_fixture;
#[path = "delivery_provisional_handoff_reconciliation_test/support.rs"]
mod support;

use core::time::Duration;
use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::decision::wait_for_history;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{candidate, seed_range};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::plan::wait_for_current;
use delivery_fixture::playback::{playing, wait_for_admission};
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::DataUsageLevel;
use support::{canonical_focus, is_cancel, next_request};

const TOTAL: u64 = 9 * 1024 * 1024;
const PREFIX: u64 = 65_536;

#[tokio::test(flavor = "current_thread")]
async fn queued_handoff_cancellation_refills_current_without_dropping_near_handoff() {
    let mut origin = ControlledOrigin::serve(TOTAL).await;
    let mut near_origin = ControlledOrigin::serve(TOTAL).await;
    let mut options = production_geometry_parallel_options();
    options.params.commitment_ms = 30_000;
    let harness = start_harness("ghostr-provisional-refill", options);
    let current = candidate("current", &origin.url_for("old-current"), Some(TOTAL), 3);
    let current_item = FocusItem {
        post: current.post.clone(),
        meta: current.meta.clone(),
    };
    let bytes = vec![7; TOTAL as usize];
    seed_range(&harness.store, &current_item, 0, &bytes).await;
    harness.handle.admit_candidate(current);
    wait_for_current(&harness.handle, "current").await;
    harness
        .handle
        .report_playback(playing("current", Duration::from_secs(20)));
    wait_for_admission(&harness.handle).await;

    harness.handle.admit_candidate(candidate(
        "next",
        &near_origin.url_for("next"),
        Some(TOTAL),
        2,
    ));
    let next = next_request("next", &mut near_origin, &harness.handle).await;
    harness
        .handle
        .admit_candidate(candidate("third", &origin.url_for("third"), Some(TOTAL), 1));
    assert!(
        tokio::time::timeout(Duration::from_millis(150), origin.next())
            .await
            .is_err(),
        "ordinary work leaves one request slot for current playback"
    );
    assert_eq!(next.range, 0..PREFIX);

    harness.handle.set_data_usage(DataUsageLevel::Balanced);
    harness.handle.update_focus(canonical_focus(&origin, TOTAL));
    wait_for_history(&harness.handle, |history| {
        history.records.iter().any(is_cancel)
    })
    .await;
    assert!(next.is_open(), "nearest handoff remains active");
    let visible = next_request("canonical current", &mut origin, &harness.handle).await;
    assert_eq!(visible.path, "/canonical-current.mp4");
    assert_eq!(visible.range.start, 0);
    assert!(next.send_byte().await);
    assert_eq!(
        harness
            .handle
            .decision_history()
            .records
            .iter()
            .filter(|record| is_cancel(record))
            .count(),
        1
    );
    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}
