//! The delivery manager publishes inventory mode transitions over a
//! watch channel so discovery can widen on hunger and stay quiet in
//! comfort (plan §5.4 unified control loop).

mod support;

use rust_lib_ghostr::engine::inventory_controller::Mode;
use rust_lib_ghostr::engine::{DataUsageLevel, EngineParams};
use rust_lib_ghostr::video::delivery_manager::{
    start_delivery_manager_with_modes, DeliveryManagerConfig, DeliveryTuning,
};
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use rust_lib_ghostr::video::playback_demand::demand_channel;
use rust_lib_ghostr::video::progressive_posts::ServablePosts;
use std::sync::Arc;
use std::time::Duration;
use support::delivery_items::{focus_now, sized_item};
use support::fixtures::{temp_directory, trusted_media_client};
use tokio::sync::Mutex;
use tokio::time::timeout;

#[tokio::test]
async fn delivery_manager_publishes_mode_transitions() {
    let root = temp_directory("ghostr-mode-watch");
    let store = Arc::new(PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0))));
    let (_demand, demand_receiver) = demand_channel();
    let config = DeliveryManagerConfig {
        store,
        client: trusted_media_client(),
        posts: ServablePosts::new(),
        stats_path: root.join("host_stats.json"),
        params: EngineParams::default(),
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
    };
    let (handle, mut modes) = start_delivery_manager_with_modes(config, demand_receiver);
    assert_eq!(*modes.borrow(), Mode::Hunger, "a fresh controller is hungry");

    // An empty focus window meets its (empty) startable target.
    handle.update_focus(focus_now(Vec::new(), 0, 0));
    timeout(Duration::from_secs(5), modes.changed())
        .await
        .expect("a mode transition should publish")
        .expect("manager should stay alive");
    assert_eq!(*modes.borrow(), Mode::Comfort);

    // Two unfetched posts drop the inventory below target minus the
    // hysteresis margin, so the controller falls back to hunger.
    let unreachable = "http://127.0.0.1:9/video.mp4";
    handle.update_focus(focus_now(
        vec![
            sized_item("aa11", unreachable, 64, 1_000),
            sized_item("bb22", unreachable, 64, 1_000),
        ],
        0,
        0,
    ));
    timeout(Duration::from_secs(5), modes.changed())
        .await
        .expect("a second transition should publish")
        .expect("manager should stay alive");
    assert_eq!(*modes.borrow(), Mode::Hunger);
    std::fs::remove_dir_all(&root).ok();
}
