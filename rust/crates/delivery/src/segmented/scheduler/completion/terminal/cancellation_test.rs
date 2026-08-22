use super::super::CompletedObject;
use super::{terminal, TerminalContext, TerminalInput};
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use ghostr_engine::adaptive::{DecisionOutcome, HlsBootstrapStage, ResourceCost};
use ghostr_engine::origin_model::NetworkClass;
use std::time::Duration;

#[test]
fn admitted_cancellation_keeps_partial_bytes_and_request_usage() {
    let finish = cancelled(Some(origin()), 37);

    assert_eq!(
        finish.outcome,
        DecisionOutcome::Cancelled {
            bytes: 37,
            elapsed_ms: 25,
        }
    );
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(37, 0, 0, 1))
    );
    assert!(finish.observation.is_none());
}

#[test]
fn pre_admission_cancellation_has_no_origin_resources() {
    let finish = cancelled(None, 0);

    assert_eq!(
        finish.outcome,
        DecisionOutcome::Cancelled {
            bytes: 0,
            elapsed_ms: 0,
        }
    );
    assert!(finish.actual_resources.is_none());
    assert!(finish.observation.is_none());
}

fn cancelled(origin: Option<OriginTelemetry>, bytes: u64) -> super::super::super::SegmentedFinish {
    let result = Err::<CompletedObject, _>(FetchFailure::cancelled(origin, bytes));
    terminal(TerminalInput {
        context: TerminalContext::new(
            "https://media.example/segment.m4s",
            HlsBootstrapStage::FirstSegment,
            ghostr_engine::ActionId::new(7),
            10,
        ),
        result: &result,
        resources: Default::default(),
    })
}

fn origin() -> OriginTelemetry {
    OriginTelemetry {
        elapsed: Duration::from_millis(25),
        ttfb: None,
        concurrency: 1,
        network_class: NetworkClass::Wifi,
    }
}
