use crate::manager::inflight::ActiveAction;
use crate::manager::plan::{planned_work, PlanInputs, PlannedWork};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::tests::adaptive_plan_fixture::playback_for;
use crate::tests::adaptive_plan_measurements::PlanMeasurements;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub(super) struct PlanScenario<'a> {
    pub(super) state: DeliveryState,
    pub(super) buffer_ms: u64,
    pub(super) bytes_per_second: u64,
    pub(super) storage: StorageSnapshot,
    pub(super) present: HashMap<ghostr_engine::PostId, Vec<ghostr_engine::ByteRange>>,
    pub(super) packet_loss_bps: u16,
    pub(super) in_flight: &'a [ActiveAction],
    pub(super) connection_capacity: usize,
}

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

fn run_with_retry_and_measurements(
    mut scenario: PlanScenario<'_>,
    retry: &RetryBook,
    measurements: PlanMeasurements,
) -> PlannedWork {
    let current = scenario.state.focus().current().cloned().expect("focus");
    scenario
        .state
        .apply_playback(playback_for(current, scenario.buffer_ms));
    let mut stats = HostStats::new();
    let sample = sample(&scenario);
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
    planned_work(
        &mut scenario.state,
        PlanInputs {
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
            storage: scenario.storage,
            connection_capacity: scenario.connection_capacity,
            connection_ceiling,
            per_authority_request_limit: connection_ceiling,
            packet_loss_bps: scenario.packet_loss_bps,
            measured_network_bytes_per_second: measurements.network_bytes_per_second,
            capacity_revision: measurements.capacity_revision,
            observed_at_ms: 1_000,
            demanded: &demanded,
        },
    )
}

fn sample(scenario: &PlanScenario<'_>) -> ThroughputSample {
    ThroughputSample::new(
        scenario.bytes_per_second,
        Duration::from_secs(1),
        1_000,
        scenario.connection_capacity,
    )
    .expect("valid throughput sample")
}
