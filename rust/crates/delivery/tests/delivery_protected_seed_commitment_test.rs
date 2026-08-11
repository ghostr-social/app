//! A protected seed remains one paid origin quantum across target shrink and promotion.

mod delivery_fixture;

use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::host_hol::SlowHost;
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_delivery::playback_demand::DemandSignal;
use ghostr_engine::{ByteRange, DataUsageLevel, EngineParams, PostId};
use std::time::Duration;

#[tokio::test]
async fn protected_seed_survives_target_shrink_and_gateway_promotion() {
    let current = serve_recording("current", vec![1; 96], hit_log()).await;
    let mut ahead = ControlledOrigin::serve(96).await;
    let blocked = SlowHost::serve().await;
    let harness = start_harness("ghostr-protected-seed-commitment", options());
    let items = vec![
        sized_item("current", &current, 96, 12_000),
        sized_item("ahead", &ahead.url, 96, 12_000),
        sized_item("blocked-1", &blocked.url("one"), 96, 12_000),
        sized_item("blocked-2", &blocked.url("two"), 96, 12_000),
    ];
    seed_range(&harness.store, &items[0], 0, &[1; 96]).await;
    harness.handle.update_focus(focus_now(items.clone(), 0, 0));

    let seed = next_request(&mut ahead).await;
    assert_eq!(seed.range, 0..96);
    send(&seed, 32).await;
    wait_for_ranges(&harness.store, "ahead", &[(0, 32)]).await;
    harness.handle.set_data_usage(DataUsageLevel::Conservative);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(seed.send_byte().await, "target shrink retained the seed");

    harness.handle.update_focus(focus_now(items, 1, 0));
    harness.demand.emit(DemandSignal {
        post: PostId::new("ahead"),
        range: ByteRange::new(33, 64),
    });
    send(&seed, 31).await;
    wait_for_ranges(&harness.store, "ahead", &[(0, 64)]).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(seed.send_byte().await, "promotion marker survived replan");
    send(&seed, 31).await;
    wait_for_ranges(&harness.store, "ahead", &[(0, 96)]).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(150), ahead.next())
            .await
            .is_err(),
        "protected seed restarted from its stored prefix"
    );

    harness.handle.clear().await.unwrap();
    blocked.release();
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

fn options() -> DeliveryOptions {
    let mut params = base_params();
    params.head_seconds = 8;
    params.head_cap_bytes = 96;
    params.chunk_bytes = 96;
    params.startable_target = 4;
    params.startable_window = 4;
    params.conservative_concurrency = 1;
    params.balanced_concurrency = 1;
    params.aggressive_concurrency = 1;
    DeliveryOptions {
        params: EngineParams { ..params },
        level: DataUsageLevel::Aggressive,
        ..DeliveryOptions::default()
    }
}
