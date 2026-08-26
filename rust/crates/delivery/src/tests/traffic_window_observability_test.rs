use super::support::temp_directory;
use crate::manager::stats::StatsKeeper;
use crate::manager::traffic::{channel, TransferKey};
use log::{Level, LevelFilter, Log, Metadata, Record};
use core::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

static LOGGER: TraceLogger = TraceLogger;

#[tokio::test]
async fn live_window_exposes_every_field_used_by_capacity_observability() {
    enable_trace_logging();
    let root = temp_directory("ghostr-traffic-observability");
    let mut keeper = StatsKeeper::load(root.join("stats.json"), Duration::ZERO).await;
    let (events, _wakes) = mpsc::unbounded_channel();
    let (publisher, inbox) = channel(events, 2);
    let started = Instant::now();
    let transfer = TransferKey::new(1);
    assert!(publisher.opened(
        transfer,
        "video.example".into(),
        Duration::from_millis(25),
        started,
    ));
    publisher.progress(transfer, 2_000, started);

    let ended = started + Duration::from_secs(1);
    let window = keeper
        .note_traffic(inbox.drain(ended))
        .expect("observable traffic window");

    assert_eq!(window.bytes(), 2_000);
    assert_eq!(window.elapsed(), Duration::from_secs(1));
    assert_eq!(window.bytes_per_second(), 2_000.0);
    assert_eq!(window.peak_active_transfers(), 1);
    assert!(window.observed_at_ms() > 0);
    assert_eq!(window.latest_ttfb(), Some(Duration::from_millis(25)));
    std::fs::remove_dir_all(root).ok();
}

fn enable_trace_logging() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(LevelFilter::Trace);
}

struct TraceLogger;

impl Log for TraceLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, _record: &Record<'_>) {}

    fn flush(&self) {}
}
