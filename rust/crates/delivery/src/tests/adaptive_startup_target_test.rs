use crate::manager::plan::{startup_seconds, StartupContext};
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use std::time::Duration;

#[test]
fn fast_stable_and_risky_hosts_receive_different_startup_reserves() {
    let mut stats = HostStats::new();
    for second in 1..=8 {
        record(&mut stats, "fast.example", 4_000_000, second);
        let slow_bytes = if second % 2 == 0 { 200_000 } else { 1_000_000 };
        record(&mut stats, "risky.example", slow_bytes, second);
    }
    stats.record_ttfb("fast.example", 100);
    stats.record_ttfb("risky.example", 1_500);

    let context = StartupContext::new(4_000_000, 8_000, 4);
    let fast = startup_seconds(&stats, "fast.example", context);
    let risky = startup_seconds(&stats, "risky.example", context);

    assert!(fast < risky);
    assert!(risky <= 6);
}

#[test]
fn an_unmeasured_connection_keeps_the_configured_startup_default() {
    let stats = HostStats::new();

    let context = StartupContext::new(4_000_000, 1, 4);
    assert_eq!(startup_seconds(&stats, "new.example", context), 4);
}

fn record(stats: &mut HostStats, host: &str, bytes: u64, second: u64) {
    let sample = ThroughputSample::new(bytes, Duration::from_secs(1), second * 1_000, 2)
        .expect("valid network sample");
    stats.record_host_throughput(host, sample);
}
