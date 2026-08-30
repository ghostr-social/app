use super::{PlanRunOptions, PlanScenario};
use crate::manager::plan::axiom_test_support::{planned_work, planned_work_with_watch};
use crate::manager::plan::{PlanInputs, PlannedWork};
use crate::manager::retry::RetryBook;
use crate::tests::adaptive_plan_fixture::playback_for;
use crate::tests::adaptive_plan_measurements::PlanMeasurements;
use ghostr_engine::host_stats::HostStats;
use std::collections::{HashMap, HashSet};

struct RunContext<'a> {
    retry: &'a RetryBook,
    measurements: PlanMeasurements,
    options: PlanRunOptions<'a>,
    stats: HostStats,
}

pub(super) fn execute(
    mut scenario: PlanScenario<'_>,
    retry: &RetryBook,
    measurements: PlanMeasurements,
    options: PlanRunOptions<'_>,
) -> PlannedWork {
    let current = scenario.state.focus().current().cloned().expect("focus");
    scenario
        .state
        .apply_playback(&playback_for(current, scenario.buffer_ms));
    let mut stats = HostStats::new();
    let sample = scenario.throughput_sample();
    stats.record_overall_throughput(sample);
    stats.record_host_throughput("media.example", sample);
    plan(
        scenario,
        RunContext {
            retry,
            measurements,
            options,
            stats,
        },
    )
}

fn plan(scenario: PlanScenario<'_>, context: RunContext<'_>) -> PlannedWork {
    let connection_ceiling = scenario.state.concurrency();
    let demanded = HashMap::new();
    let stored_totals = HashMap::new();
    let continuation_sources = HashMap::new();
    let independent_sources = HashMap::new();
    let whole_body_exhaustions = HashMap::new();
    let completed_head_probes = HashSet::new();
    let unavailable_head_probes = HashSet::new();
    let revisions = HashMap::new();
    let finalized = HashSet::new();
    let inputs = PlanInputs {
        stats: &context.stats,
        retry: context.retry,
        present: &scenario.present,
        finalized: &finalized,
        stored_totals: &stored_totals,
        continuation_sources: &continuation_sources,
        revisions: &revisions,
        independent_sources: &independent_sources,
        whole_body_exhaustions: &whole_body_exhaustions,
        completed_head_probes: &completed_head_probes,
        unavailable_head_probes: &unavailable_head_probes,
        in_flight: scenario.in_flight,
        active_head_probes: &[],
        hls_candidates: context.options.hls.unwrap_or(&[]),
        active_hls_sources: &[],
        segmented_storage_available_bytes: u64::MAX,
        storage: scenario.storage,
        connection_capacity: scenario.connection_capacity,
        hls_demand_expansion_allowed: true,
        connection_ceiling,
        per_authority_request_limit: context
            .options
            .per_authority_limit
            .unwrap_or(connection_ceiling),
        packet_loss_bps: scenario.packet_loss_bps,
        resource_feedback: Some(context.measurements.feedback(
            scenario.storage,
            scenario.bytes_per_second,
            connection_ceiling as u64,
        )),
        capacity_revision: context.measurements.capacity_revision,
        observed_at_ms: 1_000,
        demanded: &demanded,
    };
    match context.options.watch {
        Some(model) => planned_work_with_watch(&scenario.state, &inputs, model),
        None => planned_work(&scenario.state, &inputs),
    }
}
