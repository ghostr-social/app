use crate::engine::{ByteRange, PostId};
use crate::video::playback_demand::DemandSignal;
use crate::video::progressive_route::ProgressiveState;
use axum::body::Body;
use bytes::Bytes;
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
    let notify = state.store.change_notifier();
    let mut cursor = span.start;
    let mut demanded = None;
    let mut deadline = Instant::now() + state.timing.idle_timeout;
    while cursor < span.end {
        let changed = notify.notified();
        match next_chunk(&state, &key, cursor..span.end).await {
            Err(_) => return,
            Ok(Some(bytes)) => {
                cursor += bytes.len() as u64;
                deadline = Instant::now() + state.timing.idle_timeout;
                if sender.send(Ok(Bytes::from(bytes))).await.is_err() {
                    return;
                }
            }
            Ok(None) => {
                emit_demand(&state, &key, cursor..span.end, &mut demanded);
                if timeout_at(deadline, changed).await.is_err() {
                    return;
                }
            }
        }
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
