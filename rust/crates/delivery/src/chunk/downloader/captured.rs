use super::telemetry::{self, MeasuredTraffic, ObservationTiming, TrafficMeasurements};
use super::{outcome, transfer, ChunkResult, ChunkSpec};
use crate::chunk::cancel::CancelToken;
use crate::chunk::network::{prepare_network, NetworkPreparation};
use crate::chunk::sink::ChunkWrite;
use crate::chunk::traffic::ChunkTraffic;
use crate::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::HostStats;
use tokio::time::Instant;

pub(crate) struct ObservedChunk {
    pub result: anyhow::Result<ChunkResult>,
    pub origin: ghostr_engine::origin_model::OriginObservation,
}

pub async fn download<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    sink: &W,
    stats: &mut HostStats,
    cancel: &CancelToken,
    network: &NetworkThrottle,
    traffic: &mut dyn ChunkTraffic,
) -> ObservedChunk {
    let started = Instant::now();
    if spec.request.requested_bytes().is_empty() {
        let error = anyhow::anyhow!("retrieval grant must not be empty");
        return finish(
            spec,
            stats,
            Err(error),
            TrafficMeasurements::default(),
            started,
            1,
        );
    }
    let permit = match prepare_network(Some(network), spec.url, cancel).await {
        NetworkPreparation::Ready(permit) => permit,
        NetworkPreparation::Cancelled => {
            return finish(
                spec,
                stats,
                Ok(outcome::cancelled_before_request()),
                TrafficMeasurements::default(),
                started,
                1,
            )
        }
    };
    let concurrency = active_concurrency(network, spec.url);
    let mut measured = MeasuredTraffic::new(traffic);
    let result = transfer::run(spec, sink, cancel, Some(network), &mut measured).await;
    let result = record_legacy(stats, spec.url, result, started.elapsed());
    let captured = finish(
        spec,
        stats,
        result,
        measured.measurements(),
        started,
        concurrency,
    );
    drop(permit);
    captured
}

fn record_legacy(
    stats: &mut HostStats,
    url: &str,
    result: anyhow::Result<ChunkResult>,
    elapsed: std::time::Duration,
) -> anyhow::Result<ChunkResult> {
    match result {
        Ok(result) => {
            outcome::note_delivery(stats, url, &result, elapsed);
            Ok(result)
        }
        Err(error) => Err(outcome::note_failure(stats, url, error)),
    }
}

fn finish(
    spec: &ChunkSpec<'_>,
    stats: &mut HostStats,
    result: anyhow::Result<ChunkResult>,
    measured: TrafficMeasurements,
    started: Instant,
    concurrency: usize,
) -> ObservedChunk {
    let timing = ObservationTiming {
        at_ms: unix_time_ms(),
        elapsed: started.elapsed(),
        concurrency,
    };
    let origin = telemetry::observation(spec, &result, measured, timing);
    stats.origin_model_mut().observe(origin.clone());
    ObservedChunk { result, origin }
}

fn active_concurrency(network: &NetworkThrottle, url: &str) -> usize {
    let host = ghostr_engine::host_stats::host_of(url);
    network
        .active_connections()
        .into_iter()
        .find(|(active, _)| Some(active) == host.as_ref())
        .map_or(1, |(_, count)| count.max(1))
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}
