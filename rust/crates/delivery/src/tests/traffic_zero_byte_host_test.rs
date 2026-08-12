use crate::manager::traffic::{TrafficEvent, TrafficMeter, TrafficWindow, TransferKey};
use ghostr_engine::host_stats::HostStats;
use std::time::Duration;
use tokio::time::Instant;

#[test]
fn active_host_without_progress_contributes_a_silence_sample() {
    let started = Instant::now();
    let ended = started + Duration::from_secs(1);
    let mut meter = TrafficMeter::new(started, 1_800_000_000_000);
    let mut stats = HostStats::new();
    meter.observe(
        TrafficEvent::Opened {
            transfer: TransferKey::new(1),
            host: "silent.example".to_owned(),
            ttfb: Duration::from_millis(20),
            at: started,
        },
        &mut stats,
    );

    let window = meter
        .flush(TrafficWindow::new(started, ended), &mut stats)
        .expect("silence window");
    let host = stats
        .host_throughput("silent.example")
        .expect("host sample");

    assert_eq!(window.bytes(), 0);
    assert_eq!(host.bytes_per_second(), 0.0);
}
