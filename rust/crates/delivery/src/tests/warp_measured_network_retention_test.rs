use super::support::temp_directory;
use crate::manager::stats::StatsKeeper;
use crate::manager::traffic::{channel, TransferKey, SAMPLE_INTERVAL};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[tokio::test]
async fn only_a_fresh_measured_traffic_window_reaches_planning() {
    let root = temp_directory("ghostr-warp-traffic-load");
    let mut keeper = StatsKeeper::load(root.join("stats.json"), Duration::ZERO).await;
    let (events, _wakes) = mpsc::unbounded_channel();
    let (publisher, mut inbox) = channel(events, 2);
    let started = Instant::now();
    let transfer = TransferKey::new(1);
    assert!(publisher.opened(
        transfer,
        "video.example".into(),
        Duration::from_millis(25),
        started,
    ));
    publisher.progress(transfer, 2_000, started);
    let window = keeper
        .note_traffic(inbox.drain(started + Duration::from_secs(1)))
        .expect("traffic window");

    assert_eq!(
        keeper.network_load_bytes_per_second(window.observed_at_ms()),
        2_000
    );
    let stale_at = window
        .observed_at_ms()
        .saturating_add(SAMPLE_INTERVAL.as_millis() as u64)
        .saturating_add(1);
    assert_eq!(keeper.network_load_bytes_per_second(stale_at), 0);
    std::fs::remove_dir_all(root).ok();
}
