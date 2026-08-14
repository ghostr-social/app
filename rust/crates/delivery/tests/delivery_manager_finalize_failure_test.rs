mod delivery_fixture;

use delivery_fixture::demand;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness_at;
use delivery_fixture::temp_directory;
use delivery_fixture::wait::{wait_for_ranges, wait_not_servable};
use ghostr_engine::{ByteRange, DataUsageLevel, EngineParams};

#[tokio::test]
async fn failed_finalization_keeps_complete_bytes_partial_and_retires_the_attempt() {
    let root = temp_directory("ghostr-manager-finalize-failure");
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let mut options = serial_options();
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness_at(root, options);

    harness.handle.update_focus(focus_now(
        vec![sized_item("aa11", &origin, 16, 4_000)],
        0,
        5_000,
    ));
    wait_for_ranges(&harness.store, "aa11", &[(0, 8)]).await;
    std::fs::create_dir_all(harness.root.join("aa11.video")).expect("blocking completed path");
    let _demand = demand::blocked(&harness, "aa11", ByteRange::new(8, 16)).await;
    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    wait_not_servable(&harness.posts, "aa11").await;

    assert_eq!(
        harness.store.present_ranges("aa11").await.expect("ranges"),
        vec![0..16]
    );
    assert!(harness.root.join("aa11.part").exists());
    assert!(harness.root.join("aa11.video").is_dir());
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}

fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 4,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
