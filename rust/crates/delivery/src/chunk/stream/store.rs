use super::progress::StoreProgress;
use super::StreamInput;
use crate::chunk::cancel::CancelToken;
use crate::chunk::generation::OriginGeneration;
use crate::chunk::sink::{ChunkWrite, LocalStoreFailure, ResponseWriteMode};
use crate::debug::network::NetworkThrottle;
use anyhow::{Context as _, Result};

#[cfg(test)]
#[path = "store/cancellation_priority_test.rs"]
mod cancellation_priority_test;

const PACED_WRITE_BYTES: usize = 16 * 1024;

pub(super) struct StoreInput<'a, W: ChunkWrite + ?Sized> {
    generation: &'a OriginGeneration,
    sink: &'a W,
    mode: ResponseWriteMode,
    cancel: &'a CancelToken,
    network: Option<&'a NetworkThrottle>,
}

impl<'a, 'spec, W: ChunkWrite + ?Sized> From<&StreamInput<'a, 'spec, W>> for StoreInput<'a, W> {
    fn from(input: &StreamInput<'a, 'spec, W>) -> Self {
        Self {
            generation: input.generation,
            sink: input.sink,
            mode: input.mode,
            cancel: input.cancel,
            network: input.network,
        }
    }
}

pub(super) async fn store_bytes<W: ChunkWrite + ?Sized>(
    input: StoreInput<'_, W>,
    offset: u64,
    bytes: &[u8],
) -> Result<StoreProgress> {
    let quantum = write_quantum(input.network, bytes.len());
    let mut stored = 0;
    for part in bytes.chunks(quantum) {
        if store_part(&input, offset + stored, part).await? {
            stored += part.len() as u64;
        } else {
            return Ok(StoreProgress::cancelled(stored));
        }
    }
    Ok(StoreProgress::complete(stored))
}

async fn store_part<W: ChunkWrite + ?Sized>(
    input: &StoreInput<'_, W>,
    offset: u64,
    bytes: &[u8],
) -> Result<bool> {
    if pace_or_cancel(input.network, bytes.len() as u64, input.cancel).await {
        return Ok(false);
    }
    input
        .sink
        .write(input.generation, input.mode, offset, bytes)
        .await
        .context(LocalStoreFailure)
}

async fn pace_or_cancel(
    network: Option<&NetworkThrottle>,
    bytes: u64,
    cancel: &CancelToken,
) -> bool {
    if cancel.is_cancelled() {
        return true;
    }
    let Some(throttle) = network else {
        return false;
    };
    tokio::select! {
        biased;
        () = cancel.cancelled() => true,
        () = throttle.pace(bytes) => false,
    }
}

fn write_quantum(network: Option<&NetworkThrottle>, available: usize) -> usize {
    if network.is_some_and(|throttle| throttle.profile().bandwidth_kbps > 0) {
        available.clamp(1, PACED_WRITE_BYTES)
    } else {
        available.max(1)
    }
}
