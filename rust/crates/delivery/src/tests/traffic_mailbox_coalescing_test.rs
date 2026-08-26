use crate::manager::traffic::{channel, TrafficEvent, TransferKey, SAMPLE_INTERVAL};
use crate::manager::transfers::InternalEvent;
use core::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn body_chunks_coalesce_into_one_bounded_manager_wake() {
    let (events, mut wakes) = mpsc::unbounded_channel();
    let (publisher, inbox) = channel(events, 2);
    let started = Instant::now();
    let transfer = TransferKey::new(7);

    assert!(publisher.opened(
        transfer,
        "video.example".into(),
        Duration::from_millis(80),
        started,
    ));
    assert!(matches!(
        wakes.try_recv(),
        Ok(InternalEvent::TrafficChanged)
    ));
    inbox.drain(started);

    for offset in 1..10 {
        publisher.progress(transfer, 100, started + Duration::from_millis(offset));
    }
    assert!(wakes.try_recv().is_err());
    publisher.progress(transfer, 100, started + SAMPLE_INTERVAL);
    tokio::time::advance(SAMPLE_INTERVAL).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        wakes.try_recv(),
        Ok(InternalEvent::TrafficChanged)
    ));

    publisher.closed(transfer, started + SAMPLE_INTERVAL);
    assert!(wakes.try_recv().is_err());
    let batch = inbox.drain(started + SAMPLE_INTERVAL);
    assert_eq!(batch.events().len(), 2);
    assert!(matches!(
        batch.events()[0],
        TrafficEvent::Progress { bytes: 1_000, .. }
    ));
    assert!(matches!(batch.events()[1], TrafficEvent::Closed { .. }));
}
