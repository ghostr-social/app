
use crate::client_capability::{CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal, ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus};

#[test]
fn capability_generalizes_success_conservatively_and_bounds_unknown_tests() {
    let known = profile("known", "avc1", (1080, 1920));
    let smaller = profile("smaller", "AVC1", (720, 1280));
    let larger = profile("larger", "avc1", (2160, 3840));
    let alternate = profile("alternate", "hvc1", (720, 1280));
    let mut model = ClientCapabilityModel::default();
    render(&mut model, known, CapabilityAttempt::new(1, 1));

    assert!(matches!(
        model.status(9, &smaller),
        ClientCapabilityStatus::Supported { .. }
    ));
    assert_eq!(model.status(9, &larger), ClientCapabilityStatus::Unknown);
    assert_eq!(model.status(9, &alternate), ClientCapabilityStatus::Unknown);

    let attempt = CapabilityAttempt::new(1, 2);
    observe(
        &mut model,
        alternate.clone(),
        attempt,
        CapabilitySignal::Initializing,
    );
    observe(
        &mut model,
        alternate.clone(),
        attempt,
        CapabilitySignal::InconclusiveFailure,
    );
    assert_eq!(
        model.status(9, &alternate),
        ClientCapabilityStatus::Inconclusive,
    );
    assert!(!model.bounded_test_allowed(9, &alternate));
}

fn render(
    model: &mut ClientCapabilityModel,
    profile: ClientCapabilityProfile,
    attempt: CapabilityAttempt,
) {
    observe(
        model,
        profile.clone(),
        attempt,
        CapabilitySignal::Initializing,
    );
    observe(
        model,
        profile,
        attempt,
        CapabilitySignal::FirstFrameRendered,
    );
}

fn observe(
    model: &mut ClientCapabilityModel,
    profile: ClientCapabilityProfile,
    attempt: CapabilityAttempt,
    signal: CapabilitySignal,
) {
    model.observe(CapabilityObservation::new(
        9,
        attempt,
        profile,
        CapabilityEvent::new(10, signal),
    ));
}

fn profile(id: &str, codec: &str, dimensions: (u32, u32)) -> ClientCapabilityProfile {
    ClientCapabilityProfile::try_new(id, Some(codec), Some(dimensions)).expect("valid test fixture")
}
