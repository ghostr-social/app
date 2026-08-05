use crate::engine::{ByteRange, PostId};
use crate::video::playback_demand::DemandSignal;
use crate::video::progressive_route::ProgressiveState;
use axum::body::Body;
use bytes::Bytes;
use std::future::Future;
use std::io;
use std::ops::Range;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{timeout_at, Instant};
use tokio_stream::wrappers::ReceiverStream;

const READ_CHUNK_BYTES: u64 = 256 * 1024;

type ChunkSender = mpsc::Sender<Result<Bytes, io::Error>>;

/// A response body over `span` that serves what the store already holds and
/// keeps streaming as missing bytes land, emitting a demand signal while it
/// waits. The stream ends once no byte arrives within the idle timeout.
pub(crate) fn body_for_span(state: Arc<ProgressiveState>, key: String, span: Range<u64>) -> Body {
    let (sender, receiver) = mpsc::channel(4);
    tokio::spawn(pump(state, key, span, sender));
    Body::from_stream(ReceiverStream::new(receiver))
}

async fn pump(state: Arc<ProgressiveState>, key: String, span: Range<u64>, sender: ChunkSender) {
    let mut cursor = span.start;
    let mut demanded = None;
    let mut deadline = Instant::now() + state.timing.idle_timeout;
    while cursor < span.end {
        let step = advance(
            PumpIteration {
                state: &state,
                key: &key,
                remaining: cursor..span.end,
                deadline,
            },
            &sender,
            &mut demanded,
        )
        .await;
        match step {
            PumpStep::Advanced(next) => {
                cursor = next;
                deadline = Instant::now() + state.timing.idle_timeout;
            }
            PumpStep::Retry => {}
            PumpStep::Stop => return,
        }
    }
}

struct PumpIteration<'a> {
    state: &'a ProgressiveState,
    key: &'a str,
    remaining: Range<u64>,
    deadline: Instant,
}

enum PumpStep {
    Advanced(u64),
    Retry,
    Stop,
}

async fn advance(
    iteration: PumpIteration<'_>,
    sender: &ChunkSender,
    demanded: &mut Option<u64>,
) -> PumpStep {
    let notify = iteration.state.store.change_notifier();
    let changed = notify.notified();
    match next_chunk(iteration.state, iteration.key, iteration.remaining.clone()).await {
        Err(_) => PumpStep::Stop,
        Ok(Some(bytes)) => send_bytes(sender, iteration.remaining.start, bytes).await,
        Ok(None) => wait_for_bytes(iteration, changed, demanded).await,
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
    demanded: &mut Option<u64>,
) -> PumpStep {
    emit_demand(
        iteration.state,
        iteration.key,
        iteration.remaining,
        demanded,
    );
    match timeout_at(iteration.deadline, changed).await {
        Ok(()) => PumpStep::Retry,
        Err(_) => PumpStep::Stop,
    }
}

/// The next slice of bytes present at the cursor, capped for bounded memory;
/// `None` when the cursor itself is still missing from the store.
async fn next_chunk(
    state: &ProgressiveState,
    key: &str,
    remaining: Range<u64>,
) -> anyhow::Result<Option<Vec<u8>>> {
    let Some(span) = available_prefix(state, key, remaining).await? else {
        return Ok(None);
    };
    state.store.read_range(key, span).await
}

async fn available_prefix(
    state: &ProgressiveState,
    key: &str,
    remaining: Range<u64>,
) -> anyhow::Result<Option<Range<u64>>> {
    let missing = state.store.missing_within(key, remaining.clone()).await?;
    let available_end = match missing.first() {
        Some(hole) if hole.start <= remaining.start => return Ok(None),
        Some(hole) => hole.start,
        None => remaining.end,
    };
    let end = available_end.min(remaining.start.saturating_add(READ_CHUNK_BYTES));
    Ok(Some(remaining.start..end))
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
    state.demand.emit(DemandSignal {
        post: PostId::new(key),
        range: ByteRange::new(missing.start, missing.end),
    });
}
