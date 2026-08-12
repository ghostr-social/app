use crate::progressive::route::ProgressiveState;
use axum::body::Body;
use bytes::Bytes;
use ghostr_delivery::playback_demand::DemandSignal;
use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;
use ghostr_engine::{ByteRange, PostId};
use std::future::Future;
use std::io;
use std::ops::Range;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{timeout_at, Instant};
use tokio_stream::wrappers::ReceiverStream;

mod source;
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
    let mut demanded = None;
    while progress.cursor < span.end {
        let step = advance(
            PumpIteration {
                state: &state,
                source: &source,
                remaining: progress.cursor..span.end,
                deadline: progress.deadline,
            },
            &sender,
            &mut demanded,
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
    fn new(cursor: u64, timeout: std::time::Duration) -> Self {
        Self {
            cursor,
            deadline: Instant::now() + timeout,
        }
    }

    fn advance(&mut self, cursor: u64, timeout: std::time::Duration) {
        self.cursor = cursor;
        self.deadline = Instant::now() + timeout;
    }
}

async fn apply_step(
    step: PumpStep,
    progress: &mut PumpProgress,
    sender: &ChunkSender,
    timeout: std::time::Duration,
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
    Stop,
}

async fn advance(
    iteration: PumpIteration<'_>,
    sender: &ChunkSender,
    demanded: &mut Option<u64>,
) -> PumpStep {
    let notify = iteration.state.store.change_notifier();
    let changed = notify.notified();
    match next_chunk(
        iteration.state,
        iteration.source,
        iteration.remaining.clone(),
    )
    .await
    {
        Err(_) => PumpStep::Stop,
        Ok(ChunkRead::Present(bytes)) => send_bytes(sender, iteration.remaining.start, bytes).await,
        Ok(ChunkRead::Missing) => wait_for_bytes(iteration, changed, sender, demanded).await,
        Ok(ChunkRead::Superseded) => PumpStep::Superseded,
    }
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
    demanded: &mut Option<u64>,
) -> PumpStep {
    emit_demand(
        iteration.state,
        iteration.source.key(),
        iteration.remaining,
        demanded,
    );
    tokio::select! {
        biased;
        _ = sender.closed() => PumpStep::Stop,
        result = timeout_at(iteration.deadline, changed) => match result {
            Ok(()) => PumpStep::Retry,
            Err(_) => PumpStep::TimedOut,
        }
    }
}

fn emit_demand(
    state: &ProgressiveState,
    key: &str,
    missing: Range<u64>,
    demanded: &mut Option<u64>,
) {
    if *demanded == Some(missing.start) {
        return;
    }
    *demanded = Some(missing.start);
    let end = missing
        .start
        .saturating_add(PLAYBACK_SLICE_BYTES)
        .min(missing.end);
    state.demand.emit(DemandSignal {
        post: PostId::new(key),
        range: ByteRange::new(missing.start, end),
    });
}
