use super::telemetry::{self, MeasuredTraffic, ObservationTiming, TrafficMeasurements};
use super::{outcome, transfer, ChunkExecution, ChunkResult, ChunkSpec};
use crate::chunk::network::{prepare_network, NetworkPreparation};
use crate::chunk::sink::ChunkWrite;
use ghostr_engine::host_stats::HostStats;
use tokio::time::Instant;

mod early;
mod observations;
mod timing;
use timing::{is_admission_timeout, unix_time_ms};

#[derive(Debug)]
pub struct ObservedChunk {
    pub result: anyhow::Result<ChunkResult>,
    pub(crate) received_bytes: u64,
    pub(crate) origin: Option<ghostr_engine::origin_model::OriginObservation>,
    pub(crate) open_body: Option<ghostr_engine::origin_model::OpenBodyObservation>,
    pub(crate) request_started: bool,
    pub(crate) whole_body_completion: Option<crate::chunk::traffic::WholeBodyCompletion>,
    pub(crate) response_evidence: Option<super::HttpResponseEvidence>,
}

pub(super) struct CapturedTransfer {
    result: anyhow::Result<ChunkResult>,
    measured: TrafficMeasurements,
    started: Instant,
}

pub async fn download<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    execution: ChunkExecution<'_, W>,
) -> ObservedChunk {
    let started = Instant::now();
    let network_class = execution.network_class;
    if spec.request.requested_bytes().is_empty() {
        return early::invalid(spec, execution.stats, started, network_class);
    }
    let permit = match prepare_network(Some(execution.network), spec.url, execution.cancel).await {
        NetworkPreparation::Ready(permit) => permit,
        NetworkPreparation::Cancelled => {
            return early::cancelled(spec, execution.stats, started, network_class);
        }
    };
    let captured = run_transfer(spec, execution, started).await;
    drop(permit);
    captured
}

async fn run_transfer<W: ChunkWrite + ?Sized>(
    spec: &ChunkSpec<'_>,
    execution: ChunkExecution<'_, W>,
    started: Instant,
) -> ObservedChunk {
    let network_class = execution.network_class;
    let (sink, stats, cancel, network, traffic) = execution_parts(execution);
    let mut measured = MeasuredTraffic::new(traffic, network_class, spec.attempt_profile);
    let result = transfer::run(
        spec,
        transfer::TransferExecution {
            sink,
            cancel,
            network: Some(network),
            traffic: &mut measured,
        },
    )
    .await;
    let measured = measured.measurements();
    complete_transfer(
        spec,
        stats,
        CapturedTransfer {
            result,
            measured,
            started,
        },
    )
}

fn complete_transfer(
    spec: &ChunkSpec<'_>,
    stats: &mut HostStats,
    captured: CapturedTransfer,
) -> ObservedChunk {
    let CapturedTransfer {
        result,
        measured,
        started,
    } = captured;
    let result = record_legacy(stats, spec.url, result, &measured);
    finish(
        spec,
        stats,
        CapturedTransfer {
            result,
            measured,
            started,
        },
    )
}

type ExecutionParts<'a, W> = (
    &'a W,
    &'a mut HostStats,
    &'a crate::chunk::cancel::CancelToken,
    &'a crate::debug::network::NetworkThrottle,
    &'a mut dyn crate::chunk::traffic::ChunkTraffic,
);

fn execution_parts<W: ChunkWrite + ?Sized>(
    execution: ChunkExecution<'_, W>,
) -> ExecutionParts<'_, W> {
    let ChunkExecution {
        sink,
        stats,
        cancel,
        network,
        traffic,
        network_class: _,
    } = execution;
    (sink, stats, cancel, network, traffic)
}

fn record_legacy(
    stats: &mut HostStats,
    url: &str,
    result: anyhow::Result<ChunkResult>,
    measured: &TrafficMeasurements,
) -> anyhow::Result<ChunkResult> {
    if !measured.request_started() {
        return result;
    }
    let elapsed = measured.origin_elapsed().unwrap_or_default();
    match result {
        Ok(result) => {
            outcome::note_delivery(stats, url, &result, elapsed);
            Ok(result)
        }
        Err(error) if is_admission_timeout(&error) => Err(error),
        Err(error) if crate::chunk::whole_body_policy::is(&error) => {
            outcome::note_network_completion(stats, url, measured.bytes(), elapsed);
            Err(error)
        }
        Err(error) if crate::chunk::sink::is_local_store_failure(&error) => {
            if measured.whole_body_completion().is_some() {
                outcome::note_network_completion(stats, url, measured.bytes(), elapsed);
            }
            Err(error)
        }
        Err(error) => Err(outcome::note_failure(stats, url, error)),
    }
}

pub(super) fn finish(
    spec: &ChunkSpec<'_>,
    stats: &mut HostStats,
    captured: CapturedTransfer,
) -> ObservedChunk {
    let CapturedTransfer {
        result,
        measured,
        started,
    } = captured;
    let timing = ObservationTiming {
        at_ms: unix_time_ms(),
        elapsed: measured
            .origin_elapsed()
            .unwrap_or_else(|| started.elapsed()),
    };
    let (origin, open_body) = observations::record(spec, &result, &measured, timing, stats);
    ObservedChunk {
        result,
        received_bytes: measured.bytes(),
        origin,
        open_body,
        request_started: measured.request_started(),
        whole_body_completion: measured.whole_body_completion().cloned(),
        response_evidence: measured.response_evidence().cloned(),
    }
}
