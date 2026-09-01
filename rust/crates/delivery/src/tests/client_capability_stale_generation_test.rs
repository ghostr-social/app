use crate::client_capability::{
    CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal,
    ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus,
};

#[test]
fn stale_terminal_cannot_replace_the_active_player_capability_generation() {
    let profile = ClientCapabilityProfile::try_new("video", Some("avc1"), Some((720, 1_280)))
        .expect("valid test fixture");
    let mut model = ClientCapabilityModel::default();
    observe(&mut model, 8, &profile, CapabilitySignal::Initializing);

    observe(
        &mut model,
        7,
        &profile,
        CapabilitySignal::UnsupportedFailure,
    );

    assert_eq!(model.current_generation(), Some(8));
    assert_eq!(model.status(8, &profile), ClientCapabilityStatus::Testing);
    observe(
        &mut model,
        8,
        &profile,
        CapabilitySignal::FirstFrameRendered,
    );
    assert!(matches!(
        model.status(8, &profile),
        ClientCapabilityStatus::Supported { .. }
    ));
}

fn observe(
    model: &mut ClientCapabilityModel,
    generation: u64,
    profile: &ClientCapabilityProfile,
    signal: CapabilitySignal,
) {
    model.observe(CapabilityObservation::new(
        generation,
        CapabilityAttempt::new(4, 9),
        profile.clone(),
        CapabilityEvent::new(20, signal),
    ));
}
