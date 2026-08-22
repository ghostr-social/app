use crate::manager::plan::{planned_work, planned_work_with_watch, PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::tests::adaptive_plan_fixture::playback_for;
use crate::tests::adaptive_plan_measurements::PlanMeasurements;
use ghostr_engine::host_stats::HostStats;
use std::collections::{HashMap, HashSet};

mod scenario;
pub(super) use scenario::PlanScenario;

pub(super) fn run(scenario: PlanScenario<'_>) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    run_with_retry_and_measurements(scenario, &retry, PlanMeasurements::default())
}

pub(super) fn run_with_retry(scenario: PlanScenario<'_>, retry: &RetryBook) -> PlannedWork {
    run_with_retry_and_measurements(scenario, retry, PlanMeasurements::default())
}

pub(super) fn run_with_measurements(
    scenario: PlanScenario<'_>,
    measurements: PlanMeasurements,
) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    run_with_retry_and_measurements(scenario, &retry, measurements)
}

pub(super) fn run_with_watch_model(
    scenario: PlanScenario<'_>,
    model: &ghostr_engine::watch_model::WatchModel,
) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    run_with_inputs(scenario, &retry, PlanMeasurements::default(), Some(model))
}

fn run_with_retry_and_measurements(
    scenario: PlanScenario<'_>,
    retry: &RetryBook,
    measurements: PlanMeasurements,
) -> PlannedWork {
    run_with_inputs(scenario, retry, measurements, None)
}

fn run_with_inputs(
    mut scenario: PlanScenario<'_>,
    retry: &RetryBook,
    measurements: PlanMeasurements,
    watch: Option<&ghostr_engine::watch_model::WatchModel>,
) -> PlannedWork {
    let current = scenario.state.focus().current().cloned().expect("focus");
    scenario
        .state
        .apply_playback(playback_for(current, scenario.buffer_ms));
    let mut stats = HostStats::new();
    let sample = scenario.throughput_sample();
    stats.record_overall_throughput(sample);
    stats.record_host_throughput("media.example", sample);
    let connection_ceiling = scenario.state.concurrency();
    let demanded = HashMap::new();
    let stored_totals = HashMap::new();
    let continuation_sources = HashMap::new();
    let independent_sources = HashMap::new();
    let completed_head_probes = HashSet::new();
    let revisions = HashMap::new();
    let finalized = HashSet::new();
    let inputs = PlanInputs {
        stats: &stats,
        retry,
        present: &scenario.present,
        finalized: &finalized,
        stored_totals: &stored_totals,
        continuation_sources: &continuation_sources,
        revisions: &revisions,
        independent_sources: &independent_sources,
        completed_head_probes: &completed_head_probes,
        in_flight: scenario.in_flight,
        active_head_probes: &[],
        hls_candidates: &[],
        active_hls_sources: &[],
        segmented_storage_available_bytes: u64::MAX,
        storage: scenario.storage,
        connection_capacity: scenario.connection_capacity,
        connection_ceiling,
        per_authority_request_limit: connection_ceiling,
        packet_loss_bps: scenario.packet_loss_bps,
        measured_network_bytes_per_second: measurements.network_bytes_per_second,
        measured_transform_cpu_ms: None,
        capacity_revision: measurements.capacity_revision,
        observed_at_ms: 1_000,
        demanded: &demanded,
    };
    match watch {
        Some(model) => planned_work_with_watch(&mut scenario.state, inputs, model),
        None => planned_work(&mut scenario.state, inputs),
    }
}
