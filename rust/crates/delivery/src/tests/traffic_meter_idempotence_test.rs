use crate::manager::traffic::{TrafficEvent, TrafficMeter, TrafficWindow, TransferKey};
use ghostr_engine::host_stats::HostStats;
use core::time::Duration;
use tokio::time::Instant;

#[test]
fn duplicate_and_unknown_transfer_events_cannot_corrupt_a_live_window() {
    let started = Instant::now();
    let ended = started + Duration::from_secs(1);
    let mut meter = TrafficMeter::new(started, 1_800_000_000_000);
    let mut stats = HostStats::new();
    let active = TransferKey::new(1);
    let unknown = TransferKey::new(2);

    meter.observe(opened(active, "video.example", started), &mut stats);
    meter.observe(opened(active, "wrong.example", started), &mut stats);
    meter.observe(progress(unknown, 5_000, ended), &mut stats);
    meter.observe(
        TrafficEvent::Closed {
            transfer: unknown,
            at: ended,
        },
        &mut stats,
    );
    meter.observe(progress(active, 1_000, ended), &mut stats);
    meter.observe(
        TrafficEvent::Closed {
            transfer: active,
            at: ended,
        },
        &mut stats,
    );

    let window = meter
        .flush(TrafficWindow::new(started, ended), &mut stats)
        .expect("valid transfer window");
    assert_eq!(window.bytes(), 1_000);
    assert_eq!(window.latest_ttfb(), Some(Duration::from_millis(20)));
    assert!(stats.host_throughput("wrong.example").is_none());
}

fn opened(transfer: TransferKey, host: &str, at: Instant) -> TrafficEvent {
    TrafficEvent::Opened {
        transfer,
        host: host.into(),
        ttfb: Duration::from_millis(20),
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
