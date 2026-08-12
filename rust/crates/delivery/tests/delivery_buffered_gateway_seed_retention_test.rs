//! Read-ahead demand must not discard already-paid protected origin IO.

mod delivery_fixture;

use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::playback::playing;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_delivery::playback_demand::DemandSignal;
use ghostr_engine::{ByteRange, EngineParams, PostId};
use std::time::Duration;

#[tokio::test]
async fn buffered_gateway_demand_finishes_the_active_seed_before_using_its_slot() {
    let log = hit_log();
    let current = serve_recording("current", vec![1; 160], log.clone()).await;
    let mut ahead = ControlledOrigin::serve(32).await;
    let harness = start_harness("ghostr-buffered-demand-retention", options());
    let current_item = sized_item("current", &current, 160, 20_000);
    let ahead_item = sized_item("ahead", &ahead.url, 32, 4_000);
    seed_range(&harness.store, &current_item, 0, &[1; 80]).await;
    wait_for_ranges(&harness.store, "current", &[(0, 80)]).await;
    harness
        .handle
        .update_focus(focus_now(vec![current_item, ahead_item], 0, 0));
    harness
        .handle
        .report_playback(playing("current", Duration::from_secs(10)));

    let seed = next_request(&mut ahead).await;
    assert_eq!(seed.range, 0..32);
    assert!(seed.send_byte().await);
    wait_for_ranges(&harness.store, "ahead", &[(0, 1)]).await;
    let initial_current_requests = current_requests(&log);
    harness.demand.emit(DemandSignal {
        post: PostId::new("current"),
        range: ByteRange::new(80, 112),
    });

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(current_requests(&log), initial_current_requests);
    finish(seed, 31).await;
    wait_for_ranges(&harness.store, "ahead", &[(0, 32)]).await;
    tokio::time::timeout(Duration::from_secs(2), async {
        while current_requests(&log) <= initial_current_requests {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("queued current demand starts");
    assert!(
        tokio::time::timeout(Duration::from_millis(150), ahead.next())
            .await
            .is_err(),
        "completed ahead seed reopened"
    );

    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("origin request starts")
}

fn current_requests(log: &delivery_fixture::media::HitLog) -> usize {
    hits(log)
        .iter()
        .filter(|hit| hit.starts_with("current:GET:"))
        .count()
}

async fn finish(request: ActiveRequest, remaining: usize) {
    for _ in 0..remaining {
        assert!(request.send_byte().await, "seed remains active");
    }
}

fn options() -> DeliveryOptions {
    let mut params = base_params();
    params.chunk_bytes = 32;
    params.conservative_concurrency = 1;
    params.balanced_concurrency = 1;
    params.aggressive_concurrency = 1;
    DeliveryOptions {
        params: EngineParams { ..params },
        ..DeliveryOptions::default()
    }
}
