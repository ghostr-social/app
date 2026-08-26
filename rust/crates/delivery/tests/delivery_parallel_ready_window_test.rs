mod delivery_fixture;

use core::num::NonZeroUsize;
use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::playback::playing;
use delivery_fixture::start_harness;
use ghostr_engine::EngineParams;

#[tokio::test]
async fn probabilistic_reserve_retrieves_two_future_videos_in_parallel() {
    let current = ControlledOrigin::serve(32).await;
    let mut next_1 = ControlledOrigin::serve(32).await;
    let mut next_2 = ControlledOrigin::serve(32).await;
    let next_3 = ControlledOrigin::serve(32).await;
    let next_4 = ControlledOrigin::serve(32).await;
    let harness = start_harness("ghostr-parallel-ready-window", options());
    let current_item = item("current", &current);
    seed_range(&harness.store, &current_item, 0, &[7; 32]).await;
    harness.handle.update_focus(focus_now(
        vec![
            current_item,
            item("next-1", &next_1),
            item("next-2", &next_2),
            item("next-3", &next_3),
            item("next-4", &next_4),
        ],
        0,
        0,
    ));
    harness
        .handle
        .report_playback(playing("current", Duration::from_secs(20)));

    let requests = tokio::join!(
        next_request("next-1", &mut next_1),
        next_request("next-2", &mut next_2),
    );

    assert!(requests.0.send_byte().await);
    assert!(requests.1.send_byte().await);
    assert_eq!(
        harness
            .handle
            .latest_plan()
            .expect("published reserve plan")
            .plan
            .ready_reserve
            .target,
        2
    );
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn item(
    id: &'static str,
    origin: &ControlledOrigin,
) -> ghostr_delivery::delivery_events::FocusItem {
    sized_item(id, &origin.url, 32, 4_000)
}

async fn next_request(label: &str, origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .unwrap_or_else(|_| panic!("{label} starts within the ready-window deadline"))
}

fn options() -> DeliveryOptions {
    let mut options = DeliveryOptions {
        params: EngineParams {
            balanced_concurrency: 4,
            ..base_params()
        },
        ..DeliveryOptions::default()
    };
    options.tuning.max_requests_per_authority = NonZeroUsize::new(4);
    options
}
