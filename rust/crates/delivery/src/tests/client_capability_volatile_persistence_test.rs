use crate::client_capability::{
    CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal,
    ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus,
};

#[test]
fn mutable_representation_rejection_does_not_survive_restart() {
    let profile =
        ClientCapabilityProfile::try_new("mutable-url", None, None).expect("valid test fixture");
    let attempt = CapabilityAttempt::new(1, 1);
    let mut model = ClientCapabilityModel::default();
    observe(
        &mut model,
        profile.clone(),
        attempt,
        CapabilitySignal::Initializing,
    );
    observe(
        &mut model,
        profile.clone(),
        attempt,
        CapabilitySignal::UnsupportedFailure,
    );

    let restored = ClientCapabilityModel::from_state(model.state());

    assert_eq!(
        restored.status(7, &profile),
        ClientCapabilityStatus::Unknown
    );
}

fn observe(
    model: &mut ClientCapabilityModel,
    profile: ClientCapabilityProfile,
    attempt: CapabilityAttempt,
    signal: CapabilitySignal,
) {
    model.observe(CapabilityObservation::new(
        7,
        attempt,
        profile,
        CapabilityEvent::new(10, signal),
    ));
}
