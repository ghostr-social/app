//! A data-usage change raises the ceiling for policy-admitted work.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use ghostr_engine::{DataUsageLevel, EngineParams};
use tokio::time::timeout;

#[tokio::test]
async fn higher_data_usage_admits_a_distinct_planned_post() {
    let mut current = ControlledOrigin::serve(32).await;
    let mut next = ControlledOrigin::serve(32).await;
    let harness = start_harness("ghostr-delivery-config", capped_options());

    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &current.url, 32, 1_000),
            sized_item("bb22", &next.url, 32, 1_000),
        ],
        0,
        0,
    ));

    let _first = timeout(Duration::from_secs(1), current.next())
        .await
        .expect("first request starts");
    assert!(
        timeout(Duration::from_millis(100), next.next())
            .await
            .is_err(),
        "conservative mode retains one global slot"
    );

    harness.handle.set_data_usage(DataUsageLevel::Aggressive);

    timeout(Duration::from_secs(1), next.next())
        .await
        .expect("policy-admitted post uses the raised hard ceiling");
    std::fs::remove_dir_all(&harness.root).ok();
}

/// Whole-file chunks; one slot when conservative, two when aggressive.
fn capped_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 16,
            conservative_concurrency: 1,
            aggressive_concurrency: 2,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
