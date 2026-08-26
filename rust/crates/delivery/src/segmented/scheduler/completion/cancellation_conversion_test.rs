use super::{cancelled, terminal, CompletedObject, TerminalContext, TerminalInput};
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use core::time::Duration;
use ghostr_engine::adaptive::{DecisionOutcome, HlsBootstrapStage};
use ghostr_engine::origin_model::{NetworkClass, OriginOutcome};

#[test]
fn cancelling_a_superseded_response_keeps_successful_origin_evidence() {
    let origin = OriginTelemetry {
        elapsed: Duration::from_millis(25),
        ttfb: Some(Duration::from_millis(5)),
        concurrency: 1,
        network_class: NetworkClass::Wifi,
    };
    let result =
        Err::<CompletedObject, _>(cancelled(Err(FetchFailure::superseded(origin, 131_072))));
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
        finish.observation.expect("valid test fixture").outcome,
        OriginOutcome::Success
    );
}
