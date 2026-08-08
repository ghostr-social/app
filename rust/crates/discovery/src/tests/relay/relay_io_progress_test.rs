use crate::relay::io::drain_events_with_progress;
use crate::tests::scheduler_support::note_at;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn forwards_each_event_before_the_relay_stream_ends() {
    let (relay, events) = mpsc::unbounded_channel();
    let (progress, mut updates) = mpsc::channel(1);
    let draining = tokio::spawn(drain_events_with_progress(
        UnboundedReceiverStream::new(events),
        Some(progress),
    ));
    let event = note_at(40);

    relay.send(event.clone()).expect("relay stream stays open");
    assert_eq!(updates.recv().await, Some(event.clone()));
    assert!(!draining.is_finished());

    drop(relay);
    assert_eq!(draining.await.expect("drain task"), vec![event]);
}
