use crate::manager::plan::PlannedWork;
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::tests::adaptive_plan_measurements::PlanMeasurements;

mod run;
mod scenario;
use scenario::PlanRunOptions;
pub(super) use scenario::PlanScenario;

pub(super) fn run(scenario: PlanScenario<'_>) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    run::execute(
        scenario,
        &retry,
        PlanMeasurements::default(),
        Default::default(),
    )
}

pub(super) fn run_with_retry(scenario: PlanScenario<'_>, retry: &RetryBook) -> PlannedWork {
    run::execute(
        scenario,
        retry,
        PlanMeasurements::default(),
        Default::default(),
    )
}

pub(super) fn run_with_measurements(
    scenario: PlanScenario<'_>,
    measurements: PlanMeasurements,
) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    run::execute(scenario, &retry, measurements, Default::default())
}

pub(super) fn run_with_watch_model(
    scenario: PlanScenario<'_>,
    model: &ghostr_engine::watch_model::WatchModel,
) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    let options = PlanRunOptions::default().with_watch(model);
    run::execute(scenario, &retry, PlanMeasurements::default(), options)
}

pub(super) fn run_with_per_authority_limit(
    scenario: PlanScenario<'_>,
    limit: usize,
) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    let options = PlanRunOptions::default().with_per_authority_limit(limit);
    run::execute(scenario, &retry, PlanMeasurements::default(), options)
}

pub(super) fn run_with_hls(
    scenario: PlanScenario<'_>,
    candidates: &[ghostr_engine::adaptive::HlsCandidateSnapshot],
    per_authority_limit: usize,
) -> PlannedWork {
    let retry = RetryBook::new(RetryPolicy::default());
    let options = PlanRunOptions::default()
        .with_per_authority_limit(per_authority_limit)
        .with_hls(candidates);
    run::execute(scenario, &retry, PlanMeasurements::default(), options)
}

pub(super) fn run_with_hls_retry(
    scenario: PlanScenario<'_>,
    candidates: &[ghostr_engine::adaptive::HlsCandidateSnapshot],
    per_authority_limit: usize,
    retry: &RetryBook,
) -> PlannedWork {
    let options = PlanRunOptions::default()
        .with_per_authority_limit(per_authority_limit)
        .with_hls(candidates);
    run::execute(scenario, retry, PlanMeasurements::default(), options)
}
