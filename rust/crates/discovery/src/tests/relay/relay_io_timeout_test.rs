use crate::relay::io::drain_events_until;
use crate::tests::scheduler_support::note_at;
use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::test]
async fn a_stalled_sdk_stream_keeps_progress_and_ends_at_the_adapter_deadline() {
    let event = note_at(40);
    let stream = tokio_stream::iter(vec![event.clone()]).chain(tokio_stream::pending());

    let events = drain_events_until(stream, None, Duration::from_millis(5)).await;

    assert_eq!(events, [event]);
}
