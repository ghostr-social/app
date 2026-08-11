use crate::host_stats::HostStats;
use std::time::Duration;

#[test]
fn host_ttfb_overrides_the_observed_overall_fallback() {
    let mut stats = HostStats::new();
    assert_eq!(stats.expected_ttfb("unknown.example"), None);

    stats.record_ttfb("fast.example", 120);
    assert_eq!(
        stats.expected_ttfb("unknown.example"),
        Some(Duration::from_millis(120))
    );

    stats.record_ttfb("slow.example", 800);

    assert_eq!(
        stats.expected_ttfb("slow.example"),
        Some(Duration::from_millis(800))
    );
    let overall = stats.overall_ttfb().unwrap();
    assert!(overall > Duration::from_millis(120));
    assert!(overall < Duration::from_millis(800));
    assert_eq!(stats.expected_ttfb("new.example"), Some(overall));
}
