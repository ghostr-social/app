//! A data-usage change raises the allowed ceiling, not unproven concurrency.

mod delivery_fixture;

use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use ghostr_engine::{DataUsageLevel, EngineParams};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn higher_data_usage_does_not_add_a_slot_without_network_evidence() {
    let mut origin = ControlledOrigin::serve(32).await;
    let harness = start_harness("ghostr-delivery-config", capped_options());

    harness.handle.update_focus(focus_now(
        vec![
            sized_item("aa11", &origin.url, 32, 1_000),
            sized_item("bb22", &origin.url, 32, 1_000),
        ],
        0,
        0,
    ));

    let _first = timeout(Duration::from_secs(1), origin.next())
        .await
        .expect("first request starts");

    harness.handle.set_data_usage(DataUsageLevel::Aggressive);

    assert!(
        timeout(Duration::from_millis(200), origin.next())
            .await
            .is_err(),
        "same-host parallelism must wait for measured aggregate gain"
    );
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
