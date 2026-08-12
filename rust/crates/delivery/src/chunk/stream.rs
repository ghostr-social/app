//! Streams one origin response into its granted sparse-store span.

use crate::chunk::cancel::CancelToken;
use crate::chunk::downloader::ChunkSpec;
use crate::chunk::sink::ChunkWrite;
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use anyhow::{bail, Context, Result};
use reqwest::Response;
use std::time::Duration;

const PACED_WRITE_BYTES: usize = 16 * 1024;

pub(crate) struct Streamed {
    pub bytes: u64,
    pub cancelled: bool,
}

pub(crate) struct StreamInput<'a, 'spec, W: ChunkWrite + ?Sized> {
    pub response: Response,
    pub spec: &'a ChunkSpec<'spec>,
    pub sink: &'a W,
    pub cancel: &'a CancelToken,
    pub network: Option<&'a NetworkThrottle>,
    pub traffic: &'a mut dyn ChunkTraffic,
}

pub(crate) async fn stream_into<W: ChunkWrite + ?Sized>(
    mut input: StreamInput<'_, '_, W>,
) -> Result<Streamed> {
    let mut written = 0;
    while written < input.spec.range.len() {
        let Some(chunk) =
            next_or_cancel(&mut input.response, input.spec.timeouts.idle, input.cancel).await?
        else {
            return ended(written, input.cancel);
        };
        let take = capped_len(input.spec, written, &chunk) as usize;
        let quantum = write_quantum(input.network, take);
        for part in chunk[..take].chunks(quantum) {
            if pace_or_cancel(input.network, part.len() as u64, input.cancel).await {
                return Ok(stopped(written, true));
            }
            let Some(stored) = write_capped(input.spec, input.sink, written, part).await? else {
                return Ok(stopped(written, true));
            };
            written += stored;
            input.traffic.wrote(stored);
        }
    }
    Ok(stopped(written, false))
}

fn ended(written: u64, cancel: &CancelToken) -> Result<Streamed> {
    if cancel.is_cancelled() {
        return Ok(stopped(written, true));
    }
    bail!("response body ended before its advertised range")
}

fn write_quantum(network: Option<&NetworkThrottle>, available: usize) -> usize {
    match network.is_some_and(|throttle| throttle.profile().bandwidth_kbps > 0) {
        true => available.clamp(1, PACED_WRITE_BYTES),
        false => available.max(1),
    }
}

async fn next_or_cancel(
    response: &mut Response,
    idle: Duration,
    cancel: &CancelToken,
) -> Result<Option<bytes::Bytes>> {
    tokio::select! {
        _ = cancel.cancelled() => Ok(None),
        chunk = next_chunk(response, idle) => chunk,
    }
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

async fn next_chunk(response: &mut Response, idle: Duration) -> Result<Option<bytes::Bytes>> {
    tokio::time::timeout(idle, response.chunk())
        .await
        .context("chunk body read timed out")?
        .context("chunk body read failed")
}

async fn write_capped<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    written: u64,
    chunk: &[u8],
) -> Result<Option<u64>> {
    let take = capped_len(spec, written, chunk) as usize;
    let offset = spec.range.start + written;
    match sink.write(offset, &chunk[..take]).await? {
        true => Ok(Some(take as u64)),
        false => Ok(None),
    }
}

fn capped_len(spec: &ChunkSpec<'_>, written: u64, chunk: &[u8]) -> u64 {
    let remaining = (spec.range.len() - written) as usize;
    chunk.len().min(remaining) as u64
}

fn stopped(bytes: u64, cancelled: bool) -> Streamed {
    Streamed { bytes, cancelled }
}
