use crate::manager::traffic::{TrafficEvent, TrafficMeter, TrafficWindow, TransferKey};
use core::time::Duration;
use ghostr_engine::host_stats::HostStats;
use tokio::time::Instant;

#[test]
fn simultaneous_host_bytes_form_one_overall_wall_window() {
    let started = Instant::now();
    let at = started + Duration::from_secs(1);
    let unix_started_ms = 1_800_000_000_000;
    let mut meter = TrafficMeter::new(started, unix_started_ms);
    let mut stats = HostStats::new();
    let fast = TransferKey::new(1);
    let slow = TransferKey::new(2);

    meter.observe(opened(fast, "fast.example", started), &mut stats);
    meter.observe(opened(slow, "slow.example", started), &mut stats);
    meter.observe(progress(fast, 3_000, at), &mut stats);
    meter.observe(progress(slow, 1_000, at), &mut stats);
    meter.observe(TrafficEvent::Closed { transfer: fast, at }, &mut stats);
    meter.observe(TrafficEvent::Closed { transfer: slow, at }, &mut stats);
    let window = meter
        .flush(TrafficWindow::new(started, at), &mut stats)
        .expect("raw overall window");

    let overall = stats.overall_throughput().expect("overall estimate");
    assert_eq!(window.bytes(), 4_000);
    assert_eq!(window.elapsed(), Duration::from_secs(1));
    assert_eq!(window.peak_active_transfers(), 2);
    assert_eq!(window.observed_at_ms(), unix_started_ms + 1_000);
    assert_eq!(window.latest_ttfb(), Some(Duration::from_millis(50)));
    assert_eq!(overall.bytes_per_second(), 4_000.0);
    assert_eq!(overall.last_observed_at_ms(), unix_started_ms + 1_000);
    assert_eq!(
        stats
            .host_throughput("fast.example")
            .expect("valid test fixture")
            .bytes_per_second(),
        3_000.0
    );
    assert_eq!(
        stats
            .host_throughput("slow.example")
            .expect("valid test fixture")
            .bytes_per_second(),
        1_000.0
    );
}

#[test]
fn opening_ttfb_reaches_the_first_nonempty_window() {
    let started = Instant::now();
    let mut meter = TrafficMeter::new(started, 1_800_000_000_000);
    let mut stats = HostStats::new();
    let transfer = TransferKey::new(1);

    meter.observe(opened(transfer, "video.example", started), &mut stats);
    assert!(meter
        .flush(TrafficWindow::new(started, started), &mut stats)
        .is_none());
    let ended = started + Duration::from_secs(1);
    meter.observe(progress(transfer, 1_000, ended), &mut stats);
    let window = meter
        .flush(TrafficWindow::new(started, ended), &mut stats)
        .expect("first data window");

    assert_eq!(window.latest_ttfb(), Some(Duration::from_millis(50)));
}

fn opened(transfer: TransferKey, host: &str, at: Instant) -> TrafficEvent {
    TrafficEvent::Opened {
        transfer,
        host: host.into(),
        ttfb: Duration::from_millis(50),
        at,
    }
}

fn progress(transfer: TransferKey, bytes: u64, at: Instant) -> TrafficEvent {
    TrafficEvent::Progress {
        transfer,
        bytes,
        at,
    }
}
