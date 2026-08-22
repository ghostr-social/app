use crate::manager::traffic::{channel, TransferKey, SAMPLE_INTERVAL};
use crate::manager::transfers::InternalEvent;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn closed_short_transfer_still_wakes_at_the_control_boundary() {
    let (events, mut wakes) = mpsc::unbounded_channel();
    let (publisher, mut inbox) = channel(events, 1);
    let started = Instant::now();
    let transfer = TransferKey::new(1);
    assert!(publisher.opened(transfer, "video.example".into(), Duration::ZERO, started));
    publisher.progress(transfer, 100_000, started);
    publisher.closed(transfer, started + Duration::from_millis(1));
    assert!(matches!(wakes.try_recv(), Ok(InternalEvent::TrafficChanged)));
    inbox.drain(started + Duration::from_millis(1));

    tokio::time::advance(SAMPLE_INTERVAL).await;
    tokio::task::yield_now().await;

    assert!(matches!(wakes.try_recv(), Ok(InternalEvent::TrafficChanged)));
}
