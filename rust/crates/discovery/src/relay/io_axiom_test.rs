use super::*;

use tokio::time::sleep;

use tokio_stream::{Stream, StreamExt as _};

pub(crate) async fn drain_events<S>(mut stream: S) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    drain_events_with_progress(&mut stream, None).await
}

pub(crate) async fn drain_events_with_progress<S>(
    mut stream: S,
    progress: Option<EventProgress>,
) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        if let Some(progress) = &progress {
            let _ = progress.send(event.clone()).await;
        }
        events.push(event);
    }
    events
}

pub(crate) async fn drain_events_until<S>(
    mut stream: S,
    progress: Option<EventProgress>,
    wait: Duration,
) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    let deadline = sleep(wait);
    tokio::pin!(deadline);
    let mut events = Vec::new();
    loop {
        tokio::select! {
            () = &mut deadline => return events,
            event = stream.next() => match event {
                Some(event) => {
                    if let Some(progress) = &progress {
                        let _ = progress.send(event.clone()).await;
                    }
                    events.push(event);
                }
                None => return events,
            }
        }
    }
}
