use crate::progressive::route::ProgressiveState;
use axum::body::Body;
use bytes::Bytes;
use core::future::Future;
use core::ops::Range;
use core::pin::Pin;
use ghostr_delivery::playback_demand::DemandConsumer;
use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;
use ghostr_engine::{ByteRange, PostId};
use std::io;
use std::sync::Arc;
use tokio::sync::futures::Notified;
use tokio::sync::{mpsc, Notify};
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;

mod source;
#[cfg(test)]
mod tests;
pub(crate) use source::StreamSource;
use source::{next_chunk, ChunkRead};

type ChunkSender = mpsc::Sender<Result<Bytes, io::Error>>;

/// A response body over `span` that serves what the store already holds and
/// keeps streaming as missing bytes land, emitting a demand signal while it
/// waits. The body fails once no byte arrives within the idle timeout.
pub(crate) fn body_for_span(
    state: Arc<ProgressiveState>,
    source: StreamSource,
    span: Range<u64>,
) -> Body {
    let (sender, receiver) = mpsc::channel(4);
    let lease = state.store.lease(source.key());
    tokio::spawn(async move {
        let _lease = lease;
        pump(state, source, span, sender).await;
    });
    Body::from_stream(ReceiverStream::new(receiver))
}

async fn pump(
    state: Arc<ProgressiveState>,
    source: StreamSource,
    span: Range<u64>,
    sender: ChunkSender,
) {
    let mut progress = PumpProgress::new(span.start, state.timing.idle_timeout);
    let mut demand = state
        .demand
        .consumer(PostId::new(source.key()), source.binding().cloned());
    while progress.cursor < span.end {
        let step = advance(
            PumpIteration {
                state: &state,
                source: &source,
                remaining: progress.cursor..span.end,
                deadline: progress.deadline,
            },
            &sender,
            &mut demand,
        )
        .await;
        if apply_step(step, &mut progress, &sender, state.timing.idle_timeout).await {
            return;
        }
    }
}

struct PumpProgress {
    cursor: u64,
    deadline: Instant,
}

impl PumpProgress {
    fn new(cursor: u64, timeout: core::time::Duration) -> Self {
        Self {
            cursor,
            deadline: Instant::now() + timeout,
        }
    }

    fn advance(&mut self, cursor: u64, timeout: core::time::Duration) {
        self.cursor = cursor;
        self.deadline = Instant::now() + timeout;
    }
}

async fn apply_step(
    step: PumpStep,
    progress: &mut PumpProgress,
    sender: &ChunkSender,
    timeout: core::time::Duration,
) -> bool {
    match step {
        PumpStep::Advanced(next) => progress.advance(next, timeout),
        PumpStep::Retry => {}
        PumpStep::TimedOut => {
            let error = io::Error::new(io::ErrorKind::TimedOut, "progressive stream stalled");
            let _ = sender.send(Err(error)).await;
            return true;
        }
        PumpStep::Superseded => {
            let error = io::Error::other("progressive representation changed");
            let _ = sender.send(Err(error)).await;
            return true;
        }
        PumpStep::Failed(error) => {
            let _ = sender.send(Err(error)).await;
            return true;
        }
        PumpStep::Stop => return true,
    }
    false
}

struct PumpIteration<'a> {
    state: &'a ProgressiveState,
    source: &'a StreamSource,
    remaining: Range<u64>,
    deadline: Instant,
}

enum PumpStep {
    Advanced(u64),
    Retry,
    TimedOut,
    Superseded,
    Failed(io::Error),
    Stop,
}

async fn advance(
    iteration: PumpIteration<'_>,
    sender: &ChunkSender,
    demand: &mut DemandConsumer,
) -> PumpStep {
    let notify = iteration.state.store.change_notifier();
    let read = next_chunk(
        iteration.state,
        iteration.source,
        iteration.remaining.clone(),
    );
    let (chunk, changed) = read_with_armed_change(&notify, read).await;
    match chunk {
        Err(error) => PumpStep::Failed(io::Error::other(error.to_string())),
        Ok(ChunkRead::Present(bytes)) => send_bytes(sender, iteration.remaining.start, bytes).await,
        Ok(ChunkRead::Missing) => wait_for_bytes(iteration, changed, sender, demand).await,
        Ok(ChunkRead::Superseded) => PumpStep::Superseded,
    }
}

async fn read_with_armed_change<T>(
    notify: &Notify,
    read: impl Future<Output = T>,
) -> (T, Pin<Box<Notified<'_>>>) {
    let mut changed = Box::pin(notify.notified());
    changed.as_mut().enable();
    (read.await, changed)
}

async fn send_bytes(sender: &ChunkSender, cursor: u64, bytes: Vec<u8>) -> PumpStep {
    let next = cursor + bytes.len() as u64;
    match sender.send(Ok(Bytes::from(bytes))).await {
        Ok(()) => PumpStep::Advanced(next),
        Err(_) => PumpStep::Stop,
    }
}

async fn wait_for_bytes(
    iteration: PumpIteration<'_>,
    changed: impl Future<Output = ()>,
    sender: &ChunkSender,
    demand: &mut DemandConsumer,
) -> PumpStep {
    emit_demand(demand, iteration.remaining);
    wait_for_store_change(iteration.deadline, changed, sender.closed()).await
}

async fn wait_for_store_change(
    deadline: Instant,
    changed: impl Future<Output = ()>,
    closed: impl Future<Output = ()>,
) -> PumpStep {
    tokio::select! {
        biased;
        () = closed => PumpStep::Stop,
        () = tokio::time::sleep_until(deadline) => PumpStep::TimedOut,
        () = changed => PumpStep::Retry,
    }
}

fn emit_demand(demand: &mut DemandConsumer, missing: Range<u64>) {
    let end = missing
        .start
        .saturating_add(PLAYBACK_SLICE_BYTES)
        .min(missing.end);
    demand.demand(ByteRange::new(missing.start, end));
}
