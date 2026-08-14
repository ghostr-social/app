//! The scoped startup lane must contribute valid evidence for a wider base.

mod delivery_fixture;

use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::protected_capacity::{start, wait_for_bytes};
use delivery_fixture::DeliveryHarness;
use std::time::Duration;

const SAMPLE_WINDOW: Duration = Duration::from_millis(520);

#[tokio::test]
async fn protected_override_windows_admit_a_third_same_host_request() {
    let mut origin = ControlledOrigin::serve(32).await;
    let (harness, _demand) = start(&origin.url).await;

    let active = [
        next_request(&mut origin).await,
        next_request(&mut origin).await,
    ];
    expect_no_request(&mut origin).await;
    drive_learning_windows(&harness, &mut origin, &active).await;

    let third = next_request(&mut origin).await;
    assert!(!third.range.is_empty(), "trial admits useful range work");
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn drive_learning_windows(
    harness: &DeliveryHarness,
    origin: &mut ControlledOrigin,
    active: &[ActiveRequest; 2],
) {
    for window in 1..=4 {
        tokio::time::sleep(SAMPLE_WINDOW).await;
        for request in active {
            assert!(
                request.send_byte().await,
                "protected override remains active"
            );
        }
        wait_for_bytes(harness, (window * 2) as u64).await;
        if window < 4 {
            expect_no_request(origin).await;
        }
    }
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("range request in time")
}

async fn expect_no_request(origin: &mut ControlledOrigin) {
    let request = tokio::time::timeout(Duration::from_millis(100), origin.next()).await;
    if let Ok(request) = request {
        panic!(
            "base concurrency rose before enough evidence: {:?}",
            request.range
        );
    }
}
