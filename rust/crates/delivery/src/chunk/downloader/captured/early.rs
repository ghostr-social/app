use super::{finish, CapturedTransfer, ObservedChunk, TrafficMeasurements};
use crate::chunk::downloader::{outcome, ChunkResult, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::NetworkClass;
use tokio::time::Instant;

pub(super) fn invalid(
    spec: &ChunkSpec<'_>,
    stats: &mut HostStats,
    started: Instant,
    network_class: NetworkClass,
) -> ObservedChunk {
    finish(
        spec,
        stats,
        captured(
            Err(anyhow::anyhow!("retrieval grant must not be empty")),
            started,
            network_class,
        ),
    )
}

pub(super) fn cancelled(
    spec: &ChunkSpec<'_>,
    stats: &mut HostStats,
    started: Instant,
    network_class: NetworkClass,
) -> ObservedChunk {
    finish(
        spec,
        stats,
        captured(
            Ok(outcome::cancelled_before_request()),
            started,
            network_class,
        ),
    )
}

fn captured(
    result: anyhow::Result<ChunkResult>,
    started: Instant,
    network_class: NetworkClass,
) -> CapturedTransfer {
    CapturedTransfer {
        result,
        measured: TrafficMeasurements::default().with_network_class(network_class),
        started,
    }
}
