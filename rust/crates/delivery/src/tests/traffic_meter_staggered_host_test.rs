use crate::manager::traffic::{TrafficEvent, TrafficMeter, TrafficWindow, TransferKey};
use core::time::Duration;
use ghostr_engine::host_stats::HostStats;
use tokio::time::Instant;

#[test]
fn staggered_hosts_use_their_own_active_time_inside_one_overall_window() {
    let started = Instant::now();
    let halfway = started + Duration::from_millis(500);
    let ended = started + Duration::from_secs(1);
    let mut meter = TrafficMeter::new(started, 1_800_000_000_000);
    let mut stats = HostStats::new();
    let full = TransferKey::new(1);
    let late = TransferKey::new(2);

    meter.observe(opened(full, "full.example", started), &mut stats);
    meter.observe(opened(late, "late.example", halfway), &mut stats);
    meter.observe(progress(full, 500, ended), &mut stats);
    meter.observe(progress(late, 500, ended), &mut stats);
    meter.observe(
        TrafficEvent::Closed {
            transfer: full,
            at: ended,
        },
        &mut stats,
    );
    meter.observe(
        TrafficEvent::Closed {
            transfer: late,
            at: ended,
        },
        &mut stats,
    );
    let overall = meter
        .flush(TrafficWindow::new(started, ended), &mut stats)
        .expect("overall sample");

    assert_eq!(overall.bytes(), 1_000);
    assert_eq!(overall.elapsed(), Duration::from_secs(1));
    assert_eq!(rate(&stats, "full.example"), 500.0);
    assert_eq!(rate(&stats, "late.example"), 1_000.0);
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

fn rate(stats: &HostStats, host: &str) -> f64 {
    stats
        .host_throughput(host)
        .expect("valid test fixture")
        .bytes_per_second()
}
