use super::super::CompletedObject;
use super::{terminal, TerminalContext, TerminalInput};
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use core::time::Duration;
use ghostr_engine::adaptive::{HlsBootstrapStage, ResourceCost};
use ghostr_engine::origin_model::{ErrorReason, NetworkClass, OriginOutcome};

#[test]
fn timeout_failure_keeps_typed_origin_reason() {
    assert_failure_reason(ErrorReason::Timeout);
}

#[test]
fn server_failure_keeps_typed_origin_reason() {
    assert_failure_reason(ErrorReason::Http5xx);
}

#[test]
fn admitted_partial_body_failure_records_exact_network_usage() {
    let result = failure(ErrorReason::Timeout, 37);
    let finish = terminal(input(&result));
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(37, 0, 0, 1))
    );
}

#[test]
fn local_policy_failure_keeps_resources_without_poisoning_the_origin() {
    let result = failure(ErrorReason::Policy, 37);
    let finish = terminal(input(&result));

    assert!(finish.observation.is_none());
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(37, 0, 0, 1))
    );
}

#[test]
fn admitted_neutral_timeout_keeps_resources_without_poisoning_the_origin() {
    let result = Err(FetchFailure::admitted_neutral(
        anyhow::anyhow!("redirect admission expired"),
        ErrorReason::Timeout,
        telemetry(),
        37,
    ));
    let finish = terminal(input(&result));

    assert!(finish.observation.is_none());
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(37, 0, 0, 1))
    );
}

fn assert_failure_reason(expected: ErrorReason) {
    let result = failure(expected, 0);
    let finish = terminal(input(&result));
    let observed = finish.observation.expect("admitted origin observation");

    assert_eq!(observed.outcome, OriginOutcome::Failure(expected));
}

fn input(result: &Result<CompletedObject, FetchFailure>) -> TerminalInput<'_> {
    TerminalInput {
        context: TerminalContext::new(
            "https://media.example/segment.m4s",
            HlsBootstrapStage::FirstSegment,
            ghostr_engine::ActionId::new(7),
            10,
        ),
        result,
        resources: Default::default(),
    }
}

fn failure(reason: ErrorReason, bytes: u64) -> Result<CompletedObject, FetchFailure> {
    Err(FetchFailure::admitted(
        anyhow::anyhow!("typed failure fixture"),
        reason,
        telemetry(),
        bytes,
    ))
}

fn telemetry() -> OriginTelemetry {
    OriginTelemetry {
        elapsed: Duration::from_millis(25),
        ttfb: None,
        concurrency: 1,
        network_class: NetworkClass::Wifi,
    }
}
