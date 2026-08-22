use crate::client_capability::{
    CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal,
    ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus,
};

#[test]
fn release_is_not_failure_but_definitive_decode_rejection_is_unsupported() {
    let profile =
        ClientCapabilityProfile::try_new("rep", Some("avc1"), Some((1080, 1920))).unwrap();
    let released = CapabilityAttempt::new(1, 1);
    let mut model = ClientCapabilityModel::default();
    observe(
        &mut model,
        &profile,
        released,
        CapabilitySignal::Initializing,
    );
    observe(&mut model, &profile, released, CapabilitySignal::Released);
    assert_eq!(model.status(3, &profile), ClientCapabilityStatus::Unknown);
    assert!(model.bounded_test_allowed(3, &profile));

    let rejected = CapabilityAttempt::new(1, 2);
    observe(
        &mut model,
        &profile,
        rejected,
        CapabilitySignal::Initializing,
    );
    observe(
        &mut model,
        &profile,
        rejected,
        CapabilitySignal::UnsupportedFailure,
    );
    assert_eq!(
        model.status(3, &profile),
        ClientCapabilityStatus::Unsupported
    );
    assert!(!model.bounded_test_allowed(3, &profile));
}

fn observe(
    model: &mut ClientCapabilityModel,
    profile: &ClientCapabilityProfile,
    attempt: CapabilityAttempt,
    signal: CapabilitySignal,
) {
    model.observe(CapabilityObservation::new(
        3,
        attempt,
        profile.clone(),
        CapabilityEvent::new(10, signal),
    ));
}
