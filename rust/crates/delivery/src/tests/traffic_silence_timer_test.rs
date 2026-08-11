use crate::manager::traffic::{channel, TrafficMeter, TransferKey, SAMPLE_INTERVAL};
use crate::manager::transfers::InternalEvent;
use ghostr_engine::host_stats::HostStats;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn one_shot_timer_records_silence_once_and_decays_live_throughput() {
    let started = Instant::now();
    let (events, mut wakes) = mpsc::unbounded_channel();
    let (publisher, mut inbox) = channel(events, 2);
    let mut meter = TrafficMeter::new(started, 1_800_000_000_000);
    let mut stats = HostStats::new();
    let transfer = TransferKey::new(1);

    assert!(publisher.opened(
        transfer,
        "video.example".into(),
        Duration::from_millis(20),
        started,
    ));
    take_wake(&mut wakes);
    assert!(meter.apply(inbox.drain(started), &mut stats).is_none());
    publisher.progress(transfer, 1_000, started);

    advance_window().await;
    take_wake(&mut wakes);
    let live = meter
        .apply(inbox.drain(Instant::now()), &mut stats)
        .expect("live window");
    assert_eq!(live.bytes(), 1_000);
    let fast = stats.overall_throughput().unwrap();

    advance_window().await;
    take_wake(&mut wakes);
    assert!(wakes.try_recv().is_err(), "timer wake stays coalesced");
    let silent = meter
        .apply(inbox.drain(Instant::now()), &mut stats)
        .expect("silent window");
    let decayed = stats.overall_throughput().unwrap();

    assert_eq!(silent.bytes(), 0);
    assert_eq!(decayed.sample_count(), fast.sample_count() + 1);
    assert!(decayed.bytes_per_second() < fast.bytes_per_second());
}

async fn advance_window() {
    tokio::time::advance(SAMPLE_INTERVAL).await;
    tokio::task::yield_now().await;
}

fn take_wake(wakes: &mut mpsc::UnboundedReceiver<InternalEvent>) {
    assert!(matches!(
        wakes.try_recv(),
        Ok(InternalEvent::TrafficChanged)
    ));
}
