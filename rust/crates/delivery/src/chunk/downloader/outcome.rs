use super::ChunkResult;
use core::time::Duration;
use ghostr_engine::host_stats::{host_of, HostStats};

pub(super) fn range_ignored(total_bytes: Option<u64>, range_support: Option<bool>) -> ChunkResult {
    ChunkResult {
        bytes_written: 0,
        range_support,
        range_ignored: true,
        cancelled: false,
        total_bytes,
        promoted: false,
        request_started: true,
    }
}

pub(super) fn cancelled_before_request() -> ChunkResult {
    cancelled(false)
}

pub(super) fn cancelled_after_request() -> ChunkResult {
    cancelled(true)
}

fn cancelled(request_started: bool) -> ChunkResult {
    ChunkResult {
        bytes_written: 0,
        range_support: None,
        range_ignored: false,
        cancelled: true,
        total_bytes: None,
        promoted: false,
        request_started,
    }
}

pub(super) fn note_delivery(
    stats: &mut HostStats,
    url: &str,
    result: &ChunkResult,
    elapsed: Duration,
) {
    if result.cancelled {
        return;
    }
    let Some(host) = host_of(url) else { return };
    if result.bytes_written > 0 {
        stats.record_transfer(&host, result.bytes_written, elapsed);
    }
    stats.record_success(&host);
}

pub(super) fn note_network_completion(
    stats: &mut HostStats,
    url: &str,
    bytes: u64,
    elapsed: Duration,
) {
    let Some(host) = host_of(url) else { return };
    if bytes > 0 {
        stats.record_transfer(&host, bytes, elapsed);
    }
    stats.record_success(&host);
}

pub(super) fn note_failure(
    stats: &mut HostStats,
    url: &str,
    error: anyhow::Error,
) -> anyhow::Error {
    if let Some(host) = host_of(url) {
        stats.record_failure(&host);
    }
    error
}
