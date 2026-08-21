//! Streams one origin response into its granted store transaction.

use crate::chunk::cancel::CancelToken;
use crate::chunk::downloader::ChunkSpec;
use crate::chunk::generation::OriginGeneration;
use crate::chunk::sink::{ChunkWrite, ResponseWriteMode};
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use anyhow::{bail, ensure, Context, Result};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract};
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::MediaResponse;
use std::time::Duration;

mod progress;
use progress::StoreProgress;
pub(crate) use progress::Streamed;

const PACED_WRITE_BYTES: usize = 16 * 1024;

pub(crate) struct StreamInput<'a, 'spec, W: ChunkWrite + ?Sized> {
    pub response: MediaResponse,
    pub spec: &'a ChunkSpec<'spec>,
    pub generation: &'a OriginGeneration,
    pub sink: &'a W,
    pub mode: ResponseWriteMode,
    pub cancel: &'a CancelToken,
    pub network: Option<&'a NetworkThrottle>,
    pub traffic: &'a mut dyn ChunkTraffic,
}

pub(crate) async fn stream_into<W: ChunkWrite + ?Sized>(
    mut input: StreamInput<'_, '_, W>,
) -> Result<Streamed> {
    match input.spec.request {
        RetrievalRequest::FetchRange { bytes, .. } => stream_range(&mut input, bytes).await,
        RetrievalRequest::FetchWhole { contract, .. } => stream_whole(&mut input, contract).await,
    }
}

async fn stream_range<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>,
    range: ByteRange,
) -> Result<Streamed> {
    let mut written = 0;
    while written < range.len() {
        let Some(chunk) = next_input(input).await? else {
            return ended_range(written, input.cancel);
        };
        let take = chunk.len().min((range.len() - written) as usize);
        let stored = store_bytes(input, range.start + written, &chunk[..take]).await?;
        written += stored.bytes;
        if stored.cancelled {
            return Ok(stopped(written));
        }
    }
    Ok(completed_range(written))
}

async fn stream_whole<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>,
    contract: WholeBodyContract,
) -> Result<Streamed> {
    let mut written = 0_u64;
    loop {
        let Some(chunk) = next_input(input).await? else {
            return ended_whole(written, contract, input.cancel);
        };
        ensure!(
            written.saturating_add(chunk.len() as u64) <= contract.maximum_bytes(),
            "whole response exceeds its hard cap"
        );
        let stored = store_bytes(input, written, &chunk).await?;
        written += stored.bytes;
        if stored.cancelled {
            return Ok(stopped(written));
        }
    }
}

async fn store_bytes<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>,
    offset: u64,
    bytes: &[u8],
) -> Result<StoreProgress> {
    let quantum = write_quantum(input.network, bytes.len());
    let mut stored = 0;
    for part in bytes.chunks(quantum) {
        if pace_or_cancel(input.network, part.len() as u64, input.cancel).await {
            return Ok(StoreProgress::cancelled(stored));
        }
        if !input
            .sink
            .write(input.generation, input.mode, offset + stored, part)
            .await?
        {
            return Ok(StoreProgress::cancelled(stored));
        }
        stored += part.len() as u64;
        input.traffic.wrote(part.len() as u64);
    }
    Ok(StoreProgress::complete(stored))
}

async fn next_input<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>,
) -> Result<Option<bytes::Bytes>> {
    tokio::select! {
        _ = input.cancel.cancelled() => Ok(None),
        chunk = next_chunk(&mut input.response, input.spec.timeouts.idle) => chunk,
    }
}

async fn next_chunk(response: &mut MediaResponse, idle: Duration) -> Result<Option<bytes::Bytes>> {
    tokio::time::timeout(idle, response.chunk())
        .await
        .context("chunk body read timed out")?
        .context("chunk body read failed")
}

async fn pace_or_cancel(
    network: Option<&NetworkThrottle>,
    bytes: u64,
    cancel: &CancelToken,
) -> bool {
    let Some(throttle) = network else {
        return false;
    };
    tokio::select! {
        _ = cancel.cancelled() => true,
        _ = throttle.pace(bytes) => false,
    }
}

fn ended_range(written: u64, cancel: &CancelToken) -> Result<Streamed> {
    if cancel.is_cancelled() {
        return Ok(stopped(written));
    }
    bail!("response body ended before its advertised range")
}

fn ended_whole(
    written: u64,
    contract: WholeBodyContract,
    cancel: &CancelToken,
) -> Result<Streamed> {
    if cancel.is_cancelled() {
        return Ok(stopped(written));
    }
    ensure!(written > 0, "whole response body is empty");
    if let WholeBodyContract::Exact { expected_bytes } = contract {
        ensure!(written == expected_bytes, "whole response length changed");
    }
    Ok(Streamed {
        bytes: written,
        cancelled: false,
        discovered_total: Some(written),
    })
}

fn write_quantum(network: Option<&NetworkThrottle>, available: usize) -> usize {
    match network.is_some_and(|throttle| throttle.profile().bandwidth_kbps > 0) {
        true => available.clamp(1, PACED_WRITE_BYTES),
        false => available.max(1),
    }
}

fn stopped(bytes: u64) -> Streamed {
    Streamed {
        bytes,
        cancelled: true,
        discovered_total: None,
    }
}

fn completed_range(bytes: u64) -> Streamed {
    Streamed {
        bytes,
        cancelled: false,
        discovered_total: None,
    }
}
