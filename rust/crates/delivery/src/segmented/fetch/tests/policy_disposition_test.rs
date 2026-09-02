use super::super::telemetry::OriginTelemetry;
use super::super::FetchFailure;
use core::time::Duration;
use ghostr_engine::origin_model::{ErrorReason, NetworkClass};

const POLICY_ERROR: &str = "encrypted HLS is not supported";

fn origin() -> OriginTelemetry {
    OriginTelemetry {
        elapsed: Duration::from_millis(25),
        ttfb: None,
        concurrency: 1,
        network_class: NetworkClass::Wifi,
    }
}

#[test]
fn media_policy_rejection_is_terminal_on_the_first_response() {
    let failure = FetchFailure::admitted(
        anyhow::anyhow!(POLICY_ERROR),
        ErrorReason::Policy,
        origin(),
        0,
    );

    assert!(
        failure.is_local_terminal(),
        "a deterministic policy rejection must not be retried"
    );
    assert!(failure.retry_class().is_none());
    assert!(
        !failure.records_origin_evidence(),
        "policy is a local verdict, not origin behavior"
    );
}
