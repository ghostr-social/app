//! A partially received range remains the sole owner of those origin bytes
//! across promotion and gateway demand replans.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::demand;
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::plan::wait_for_current;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::{ByteRange, DataUsageLevel};

#[tokio::test]
async fn promotion_never_restarts_bytes_owned_by_an_open_request() {
    let current = serve_recording("current", vec![1; 96], hit_log()).await;
    let mut ahead = ControlledOrigin::serve(96).await;
    let harness = start_harness("ghostr-protected-seed-commitment", options());
    let items = vec![
        sized_item("current", &current, 96, 12_000),
        sized_item("ahead", &ahead.url, 96, 12_000),
    ];
    seed_range(&harness.store, &items[0], 0, &[1; 96]).await;
    harness.handle.update_focus(focus_now(items.clone(), 0, 0));

    let seed = next_request(&mut ahead).await;
    assert_eq!(seed.range, 0..64);
    send(&seed, 32).await;
    wait_for_ranges(&harness.store, "ahead", &[(0, 32)]).await;

    harness.handle.update_focus(focus_now(items, 1, 0));
    wait_for_current(&harness.handle, "ahead").await;
    assert!(seed.is_open(), "focus promotion retains the seed body");
    let _demand = demand::blocked(&harness, "ahead", ByteRange::new(0, 96)).await;
    expect_no_request(&mut ahead).await;
    send(&seed, 32).await;
    drop(seed);
    let tail = next_request(&mut ahead).await;
    assert_eq!(tail.range, 64..96);
    send(&tail, 32).await;
    drop(tail);
    wait_for_ranges(&harness.store, "ahead", &[(0, 96)]).await;
    expect_no_request(&mut ahead).await;

    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("protected seed starts")
}

async fn send(request: &ActiveRequest, bytes: usize) {
    for _ in 0..bytes {
        assert!(request.send_byte().await, "seed remains accepted");
    }
}

async fn expect_no_request(origin: &mut ControlledOrigin) {
    if let Ok(request) = tokio::time::timeout(Duration::from_millis(150), origin.next()).await {
        panic!(
            "origin received {:?} while the planned bytes were owned",
            request.range
        );
    }
}

fn options() -> DeliveryOptions {
    let mut params = base_params();
    params.chunk_bytes = 64;
    params.conservative_concurrency = 1;
    params.balanced_concurrency = 1;
    params.aggressive_concurrency = 1;
    DeliveryOptions {
        params,
        level: DataUsageLevel::Aggressive,
        ..DeliveryOptions::default()
    }
}
