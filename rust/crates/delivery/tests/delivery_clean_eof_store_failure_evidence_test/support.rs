use super::delivery_fixture::decision::wait_for_history;
use super::delivery_fixture::items::unsized_item;
use super::delivery_fixture::options::DeliveryOptions;
use super::delivery_fixture::stats::wait_for;
use super::delivery_fixture::DeliveryHarness;
use core::time::Duration;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::host_stats::host_of;

const WAIT_LIMIT: Duration = Duration::from_secs(30);

pub(super) fn options() -> DeliveryOptions {
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 3;
    options.tuning.retry.base = Duration::from_millis(5);
    options.tuning.retry.max = Duration::from_millis(5);
    options.tuning.store_pressure_pause = WAIT_LIMIT;
    options
}

pub(super) fn hashed_item(url: &str) -> FocusItem {
    let mut item = unsized_item("aa11", url);
    item.meta.sha256 = Some(super::DIGEST.into());
    item
}

pub(super) fn block_publication(harness: &DeliveryHarness) {
    std::fs::create_dir(harness.root.join("aa11.response.ranges")).expect("valid test fixture");
}

pub(super) async fn assert_failure_evidence(harness: &DeliveryHarness, origin: &str) {
    wait_for_history(&harness.handle, super::learned_after_failed_whole).await;
    let host = host_of(origin).expect("valid test fixture");
    let stats = wait_for(&harness.root.join("host_stats.json"), |stats| {
        stats.host_throughput(&host).is_some()
    })
    .await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(
        stats.failure_ratio(&host),
        0.0,
        "local storage failure is not an origin failure"
    );
    assert!(
        !harness.root.join("aa11.video").exists(),
        "storage failure leaves no completed object"
    );
    assert!(
        !harness.root.join("aa11.verified").exists(),
        "storage failure leaves no verification marker"
    );
}
