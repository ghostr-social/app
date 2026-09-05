use crate::manager::traffic::{channel, TrafficMeter, TransferKey};
use core::time::Duration;
use ghostr_engine::host_stats::HostStats;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn rate_lease_wait_is_not_measured_as_a_slow_origin_or_a_new_connection() {
    let (events, _wakes) = tokio::sync::mpsc::unbounded_channel();
    let (publisher, inbox) = channel(events, 4);
    let at = Instant::now();
    let mut meter = TrafficMeter::new(at, 0);
    let mut stats = HostStats::new();
    let transfer = TransferKey::new(1);
    publisher.opened(transfer, "video.example".into(), Duration::from_millis(50), at);
    publisher.progress(transfer, 1000, at + Duration::from_millis(100));
    publisher.closed(transfer, at + Duration::from_millis(100));
    let resumed = transfer.next_window();
    assert!(publisher.resumed(resumed, "video.example".into(), at + Duration::from_millis(900)));
    publisher.progress(resumed, 1000, at + Duration::from_secs(1));
    publisher.closed(resumed, at + Duration::from_secs(1));
    let window = meter.apply(inbox.drain(at + Duration::from_secs(1)), &mut stats).expect("traffic");
    assert_eq!(window.elapsed(), Duration::from_millis(200));
    assert_eq!(window.bytes(), 2000);
    assert_eq!(window.peak_active_transfers(), 1);
    assert_eq!(stats.overall_ttfb(), Some(Duration::from_millis(50)));
}
