//! External nostr-sdk network calls behind the relay-pool owner.

use nostr_sdk::{Event, Filter};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
#[cfg(test)]
use tokio::time::sleep;
#[cfg(test)]
use tokio_stream::{Stream, StreamExt};

use crate::relay::health::RelayAdmissionBatch;
use crate::retrieval_types::EventProgress;

mod result;
mod sdk;

pub use result::RelayReadResult;
pub(crate) use sdk::SdkRelayIo;

pub type RelayIoFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>;

pub struct RelayReadIo {
    pub(crate) relays: Vec<String>,
    pub(crate) filter: Filter,
    pub(crate) timeout: Duration,
    pub(crate) progress: Option<EventProgress>,
    pub(crate) admissions: Option<RelayAdmissionBatch>,
}

pub struct RelayBroadcastIo {
    pub(crate) relays: Vec<String>,
    pub(crate) event: Event,
}

pub trait RelayIo: Send + Sync {
    fn read(&self, request: RelayReadIo) -> RelayIoFuture<'_, RelayReadResult>;
    fn broadcast(&self, request: RelayBroadcastIo) -> RelayIoFuture<'_, ()>;
}

#[cfg(test)]
pub(crate) async fn drain_events<S>(mut stream: S) -> Vec<Event>
where
    S: Stream<Item = Event> + Unpin,
{
    drain_events_with_progress(&mut stream, None).await
}

#[cfg(test)]
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

#[cfg(test)]
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
            _ = &mut deadline => return events,
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
