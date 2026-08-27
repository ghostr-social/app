mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;

#[tokio::test]
async fn an_unresolved_reserve_forecast_is_not_scored_as_its_own_outcome() {
    let current = ControlledOrigin::serve(32).await;
    let next = ControlledOrigin::serve(32).await;
    let harness = start_harness("ghostr-readiness-attribution", DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("current", &current.url, 32, 1_000),
            sized_item("next", &next.url, 32, 1_000),
        ],
        0,
        0,
    ));

    wait_for_forecast(&harness.handle).await;
    let evidence = evidence(&harness.handle);
    assert!(
        evidence["budget"]["observations"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );
    assert_eq!(evidence["readiness"]["on_time_readiness_samples"], 0);
    assert_eq!(
        evidence["readiness"]["on_time_readiness_calibration_bps"],
        0
    );

    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_forecast(handle: &DeliveryHandle) {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if handle.latest_plan().is_some_and(|entry| {
                let reserve = entry.plan.ready_reserve;
                reserve.target > 0 && reserve.recovery_horizon_ms > 0
            }) {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("reserve forecast publication");
}

fn evidence(handle: &DeliveryHandle) -> serde_json::Value {
    let json = handle.evidence_page_json(0, 0).expect("delivery evidence");
    serde_json::from_str::<serde_json::Value>(&json).expect("evidence JSON")["evaluation"].clone()
}
