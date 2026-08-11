//! Ranged chunk downloader (plan Phase 1 step 5): fetches one granted
//! byte range with an HTTP `Range` GET, streams it into the partial
//! range store, and feeds the per-host performance model. Honors
//! cooperative cancellation so scroll-past can abandon transfers while
//! keeping the bytes already fetched.

use crate::chunk::cancel::CancelToken;
use crate::chunk::network::{prepare_network, NetworkPreparation};
use crate::chunk::response::{classify, RangeReply};
use crate::chunk::sink::ChunkWrite;
use crate::chunk::stream::{stream_into, Streamed};
use crate::chunk::traffic::{ChunkTraffic, NoopTraffic};
use crate::debug::network::NetworkThrottle;
use anyhow::{ensure, Result};
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::ByteRange;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use std::time::Duration;
use tokio::time::Instant;

mod opened;
pub use crate::chunk::sink::ChunkSink;
use opened::send_ranged;

/// One granted transfer: which range of which URL to fetch.
pub struct ChunkSpec<'a> {
    pub client: &'a dyn MediaHttpRequests,
    pub url: &'a str,
    pub range: ByteRange,
    pub timeouts: TransferTimeouts,
}

/// What one chunk transfer accomplished.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkResult {
    pub bytes_written: u64,
    pub accept_ranges: bool,
    pub cancelled: bool,
    pub total_bytes: Option<u64>,
    pub(crate) request_started: bool,
}

/// Downloads one granted chunk into the store. Accepts `206 Partial
/// Content`, and `200 OK` only for grants starting at byte zero (the
/// body is a full-file stream, capped at the grant). A `200` at a
/// nonzero offset writes nothing and reports `accept_ranges: false` so
/// the engine reclassifies the video as all-or-nothing. Completed and
/// cancelled transfers record host throughput and success; failures
/// record a host failure. Cancelled transfers keep persisted bytes.
pub async fn download_chunk_throttled(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: &NetworkThrottle,
) -> Result<ChunkResult> {
    run_download(spec, sink, stats, cancel, Some(network), &mut NoopTraffic).await
}

pub(crate) async fn download_chunk_observed<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: &NetworkThrottle,
    traffic: &mut dyn ChunkTraffic,
) -> Result<ChunkResult> {
    run_download(spec, sink, stats, cancel, Some(network), traffic).await
}

async fn run_download<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: Option<&NetworkThrottle>,
    traffic: &mut dyn ChunkTraffic,
) -> Result<ChunkResult> {
    ensure!(!spec.range.is_empty(), "chunk grant must not be empty");
    let started = Instant::now();
    let _permit = match prepare_network(network, spec.url, cancel).await {
        NetworkPreparation::Ready(permit) => permit,
        NetworkPreparation::Cancelled => return Ok(cancelled_before_request()),
    };
    match transfer(spec, sink, cancel, network, traffic).await {
        Ok(result) => {
            note_delivery(stats, spec.url, &result, started.elapsed());
            Ok(result)
        }
        Err(error) => Err(note_failure(stats, spec.url, error)),
    }
}

async fn transfer<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    cancel: &CancelToken,
    network: Option<&NetworkThrottle>,
    traffic: &mut dyn ChunkTraffic,
) -> Result<ChunkResult> {
    let opened = send_ranged(spec).await?;
    traffic.opened(opened.ttfb);
    let response = opened.response;
    let full_length = response.content_length();
    match classify(&response, spec.range)? {
        RangeReply::Ignored => Ok(range_ignored(full_length)),
        RangeReply::Partial { range, total } => {
            let returned = ChunkSpec {
                client: spec.client,
                url: spec.url,
                range,
                timeouts: spec.timeouts,
            };
            completed(
                stream_into(response, &returned, sink, cancel, network, traffic).await?,
                true,
                total,
            )
        }
        RangeReply::FullBody => completed(
            stream_into(response, spec, sink, cancel, network, traffic).await?,
            false,
            full_length,
        ),
    }
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
