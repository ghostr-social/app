use super::{is_admission_timeout, telemetry, ObservationTiming, TrafficMeasurements};
use crate::chunk::downloader::{ChunkResult, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::{OpenBodyObservation, OriginObservation};

pub(super) fn record(
    spec: &ChunkSpec<'_>,
    result: &anyhow::Result<ChunkResult>,
    measured: &TrafficMeasurements,
    timing: ObservationTiming,
    stats: &mut HostStats,
) -> (Option<OriginObservation>, Option<OpenBodyObservation>) {
    let origin = (!ignore_origin(result, measured)).then(|| {
        let item = telemetry::observation(spec, result, measured, timing);
        stats.origin_model_mut().observe(&item);
        item
    });
    let open_body = telemetry::open_body_observation(spec, result, measured, timing.at_ms);
    if let Some(item) = &open_body {
        stats.origin_model_mut().observe_open_body(item);
    }
    (origin, open_body)
}

fn ignore_origin(result: &anyhow::Result<ChunkResult>, measured: &TrafficMeasurements) -> bool {
    !measured.request_started()
        || result.as_ref().err().is_some_and(is_admission_timeout)
        || local_before_network_completion(result, measured)
}

fn local_before_network_completion(
    result: &anyhow::Result<ChunkResult>,
    measured: &TrafficMeasurements,
) -> bool {
    result
        .as_ref()
        .err()
        .is_some_and(crate::chunk::sink::is_local_store_failure)
        && measured.whole_body_completion().is_none()
}
