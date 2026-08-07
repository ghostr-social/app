//! Ranged chunk downloader (plan Phase 1 step 5): fetches one granted
//! byte range with an HTTP `Range` GET, streams it into the partial
//! range store, and feeds the per-host performance model. Honors
//! cooperative cancellation so scroll-past can abandon transfers while
//! keeping the bytes already fetched.

use crate::engine::host_stats::{host_of, HostStats};
use crate::engine::ByteRange;
use crate::video::chunk_cancel::CancelToken;
use crate::video::chunk_network::{prepare_network, NetworkPreparation};
use crate::video::chunk_response::{classify, RangeReply};
use crate::video::chunk_stream::{stream_into, Streamed};
use crate::video::debug_network::NetworkThrottle;
use crate::video::outbound_media_client::MediaHttpClient;
use crate::video::partial_range_store::PartialRangeStore;
use crate::video::transfer_timeouts::TransferTimeouts;
use anyhow::{ensure, Context, Result};
use reqwest::header::RANGE;
use reqwest::Response;
use std::time::Duration;
use tokio::time::Instant;

/// One granted transfer: which range of which URL to fetch.
pub struct ChunkSpec<'a> {
    pub client: &'a MediaHttpClient,
    pub url: &'a str,
    pub range: ByteRange,
    pub timeouts: TransferTimeouts,
}

/// Where fetched bytes land.
pub struct ChunkSink<'a> {
    pub store: &'a PartialRangeStore,
    pub key: &'a str,
}

/// What one chunk transfer accomplished.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkResult {
    pub bytes_written: u64,
    pub accept_ranges: bool,
    pub cancelled: bool,
    pub total_bytes: Option<u64>,
    pub request_started: bool,
}

/// Downloads one granted chunk into the store. Accepts `206 Partial
/// Content`, and `200 OK` only for grants starting at byte zero (the
/// body is a full-file stream, capped at the grant). A `200` at a
/// nonzero offset writes nothing and reports `accept_ranges: false` so
/// the engine reclassifies the video as all-or-nothing. Completed and
/// cancelled transfers record host throughput and success; failures
/// record a host failure. Cancelled transfers keep persisted bytes.
pub async fn download_chunk(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    stats: &mut HostStats,
    cancel: &CancelToken,
) -> Result<ChunkResult> {
    run_download(spec, sink, stats, cancel, None).await
}

pub(crate) async fn download_chunk_throttled(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: &NetworkThrottle,
) -> Result<ChunkResult> {
    run_download(spec, sink, stats, cancel, Some(network)).await
}

async fn run_download(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: Option<&NetworkThrottle>,
) -> Result<ChunkResult> {
    ensure!(!spec.range.is_empty(), "chunk grant must not be empty");
    let started = Instant::now();
    let _permit = match prepare_network(network, spec.url, cancel).await {
        NetworkPreparation::Ready(permit) => permit,
        NetworkPreparation::Cancelled => return Ok(cancelled_before_request()),
    };
    match transfer(spec, sink, cancel, network).await {
        Ok(result) => {
            note_delivery(stats, spec.url, &result, started.elapsed());
            Ok(result)
        }
        Err(error) => Err(note_failure(stats, spec.url, error)),
    }
}

async fn transfer(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    cancel: &CancelToken,
    network: Option<&NetworkThrottle>,
) -> Result<ChunkResult> {
    let response = send_ranged(spec).await?;
    let full_length = response.content_length();
    match classify(&response, spec.range)? {
        RangeReply::Ignored => Ok(range_ignored(full_length)),
        RangeReply::Partial { total } => completed(
            stream_into(response, spec, sink, cancel, network).await?,
            true,
            total,
        ),
        RangeReply::FullBody => completed(
            stream_into(response, spec, sink, cancel, network).await?,
            false,
            full_length,
        ),
    }
}

async fn send_ranged(spec: &ChunkSpec<'_>) -> Result<Response> {
    let header = format!("bytes={}-{}", spec.range.start, spec.range.end - 1);
    let request = spec.client.get(spec.url)?.header(RANGE, header);
    let response = tokio::time::timeout(spec.timeouts.headers, request.send())
        .await
        .context("chunk response headers timed out")?
        .context("chunk request failed")?;
    response
        .error_for_status()
        .context("chunk request rejected")
}

fn completed(
    streamed: Streamed,
    accept_ranges: bool,
    total_bytes: Option<u64>,
) -> Result<ChunkResult> {
    Ok(ChunkResult {
        bytes_written: streamed.bytes,
        accept_ranges,
        cancelled: streamed.cancelled,
        total_bytes,
        request_started: true,
    })
}

fn range_ignored(total_bytes: Option<u64>) -> ChunkResult {
    ChunkResult {
        bytes_written: 0,
        accept_ranges: false,
        cancelled: false,
        total_bytes,
        request_started: true,
    }
}

fn cancelled_before_request() -> ChunkResult {
    ChunkResult {
        bytes_written: 0,
        accept_ranges: false,
        cancelled: true,
        total_bytes: None,
        request_started: false,
    }
}

fn note_delivery(stats: &mut HostStats, url: &str, result: &ChunkResult, elapsed: Duration) {
    let Some(host) = host_of(url) else { return };
    if result.bytes_written > 0 {
        stats.record_transfer(&host, result.bytes_written, elapsed);
    }
    stats.record_success(&host);
}

fn note_failure(stats: &mut HostStats, url: &str, error: anyhow::Error) -> anyhow::Error {
    if let Some(host) = host_of(url) {
        stats.record_failure(&host);
    }
    error
}
