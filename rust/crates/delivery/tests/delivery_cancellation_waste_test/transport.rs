use super::{
    delivery_fixture::DeliveryHarness, range_fixture::cancellation::CancellableOrigin, PREFIX,
    TOTAL,
};
use core::{sync::atomic::Ordering, time::Duration};

pub(super) async fn assert_transport_stops(old: &CancellableOrigin, harness: &DeliveryHarness) {
    let before = harness
        .store
        .present_ranges("old")
        .await
        .expect("old ranges");
    assert_eq!(
        before,
        vec![0..PREFIX as u64],
        "versioned prefix survives cancellation"
    );
    old.release.notify_one();
    tokio::time::timeout(Duration::from_secs(2), old.finished.notified())
        .await
        .expect("cancelled origin closes");
    let sent = old.bytes_sent.load(Ordering::SeqCst);
    assert!(sent < TOTAL, "cancelled origin drained {sent} bytes");
    let after = harness
        .store
        .present_ranges("old")
        .await
        .expect("old ranges");
    assert_eq!(after, before, "cancelled tail cannot become durable");
}
