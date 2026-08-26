use super::super::CompletedObject;
use super::{terminal, TerminalContext, TerminalInput};
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use core::time::Duration;
use ghostr_engine::adaptive::{DecisionOutcome, HlsBootstrapStage, ResourceCost};
use ghostr_engine::origin_model::{NetworkClass, OriginOutcome};

#[test]
fn local_preparation_cancellation_preserves_successful_origin_evidence() {
    let origin = OriginTelemetry {
        elapsed: Duration::from_millis(25),
        ttfb: Some(Duration::from_millis(5)),
        concurrency: 1,
        network_class: NetworkClass::Wifi,
    };
    let result = Err::<CompletedObject, _>(FetchFailure::cancelled_after_response(origin, 131_072));
    let finish = terminal(TerminalInput {
        context: TerminalContext::new(
            "https://media.example/segment.m4s",
            HlsBootstrapStage::FirstSegment,
            ghostr_engine::ActionId::new(7),
            10,
        ),
        result: &result,
        resources: Default::default(),
    });

    assert!(matches!(finish.outcome, DecisionOutcome::Cancelled { .. }));
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(131_072, 0, 0, 1))
    );
    assert_eq!(
        finish.observation.expect("valid test fixture").outcome,
        OriginOutcome::Success
    );
}
