use super::super::CompletedObject;
use super::{terminal, TerminalContext, TerminalInput};
use crate::segmented::fetch::OriginTelemetry;
use core::time::Duration;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::origin_model::{NetworkClass, OriginOutcome};

#[test]
fn subthreshold_hls_object_keeps_success_and_ttfb_without_throughput_evidence() {
    let result = Ok(completed(65_535, 101, 100));
    let observation = terminal(input(&result))
        .observation
        .expect("valid test fixture");

    assert_eq!(observation.outcome, OriginOutcome::Success);
    assert_eq!(observation.ttfb_ms, Some(100));
    assert_eq!(observation.throughput_bps, None);
}

#[test]
fn reliable_hls_sample_uses_body_time_after_ttfb() {
    let result = Ok(completed(65_536, 101, 100));
    let observation = terminal(input(&result))
        .observation
        .expect("valid test fixture");

    assert_eq!(observation.throughput_bps, Some(524_288_000));
}

fn input(
    result: &Result<CompletedObject, crate::segmented::fetch::FetchFailure>,
) -> TerminalInput<'_> {
    TerminalInput {
        context: TerminalContext::new(
            "https://media.example/init.mp4",
            HlsBootstrapStage::Initialization,
            ghostr_engine::ActionId::new(7),
            10,
        ),
        result,
        resources: Default::default(),
    }
}

fn completed(bytes: u64, elapsed_ms: u64, ttfb_ms: u64) -> CompletedObject {
    CompletedObject {
        bytes,
        telemetry: OriginTelemetry {
            elapsed: Duration::from_millis(elapsed_ms),
            ttfb: Some(Duration::from_millis(ttfb_ms)),
            concurrency: 1,
            network_class: NetworkClass::Wifi,
        },
    }
}
