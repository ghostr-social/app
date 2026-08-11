//! Streams one origin response into its granted sparse-store span.

use crate::chunk::cancel::CancelToken;
use crate::chunk::downloader::ChunkSpec;
use crate::chunk::sink::ChunkWrite;
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use anyhow::{bail, Context, Result};
use reqwest::Response;
use std::time::Duration;
use tokio::time::Instant;

pub(crate) struct Streamed {
    pub bytes: u64,
    pub cancelled: bool,
}

pub(crate) async fn stream_into<W: ChunkWrite + ?Sized>(
    mut response: Response,
    spec: &ChunkSpec<'_>,
    sink: &W,
    cancel: &CancelToken,
    network: Option<&NetworkThrottle>,
    traffic: &mut dyn ChunkTraffic,
) -> Result<Streamed> {
    let mut written = 0;
    let started = Instant::now();
    while written < spec.range.len() {
        let Some(chunk) = next_or_cancel(&mut response, spec.timeouts.idle, cancel).await? else {
            if cancel.is_cancelled() {
                return Ok(stopped(written, true));
            }
            bail!("response body ended before its advertised range");
        };
        let received = written + capped_len(spec, written, &chunk);
        if pace_or_cancel(network, received, started, cancel).await {
            return Ok(stopped(written, true));
        }
        let Some(stored) = write_capped(spec, sink, written, &chunk).await? else {
            return Ok(stopped(written, true));
        };
        written += stored;
        traffic.wrote(stored);
    }
    Ok(stopped(written, false))
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
    started: Instant,
    cancel: &CancelToken,
) -> bool {
    let Some(throttle) = network else {
        return false;
    };
    tokio::select! {
        _ = cancel.cancelled() => true,
        _ = throttle.pace(bytes, started) => false,
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
