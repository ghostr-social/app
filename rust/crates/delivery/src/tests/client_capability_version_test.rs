
use crate::client_capability::{CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal, ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus};

#[test]
fn first_frame_evidence_is_versioned_and_persisted() {
    let profile = profile("representation-a", "avc1", (1080, 1920));
    let attempt = CapabilityAttempt::new(7, 11);
    let mut model = ClientCapabilityModel::default();
    model.observe(CapabilityObservation::new(
        41,
        attempt,
        profile.clone(),
        CapabilityEvent::new(100, CapabilitySignal::Initializing),
    ));
    assert_eq!(model.status(41, &profile), ClientCapabilityStatus::Testing);

    model.observe(CapabilityObservation::new(
        41,
        attempt,
        profile.clone(),
        CapabilityEvent::new(350, CapabilitySignal::FirstFrameRendered),
    ));
    assert_eq!(
        model.status(41, &profile),
        ClientCapabilityStatus::Supported {
            p95_first_frame_us: 250,
        },
    );

    let restored = ClientCapabilityModel::from_state(model.state());
    assert_eq!(restored.current_generation(), None);
    assert_eq!(restored.status(41, &profile), model.status(41, &profile));
    assert_eq!(
        restored.status(42, &profile),
        ClientCapabilityStatus::Unknown,
    );
    let mut restored = restored;
    restored.observe(CapabilityObservation::new(
        41,
        CapabilityAttempt::new(8, 12),
        profile.clone(),
        CapabilityEvent::new(400, CapabilitySignal::Initializing),
    ));
    assert_eq!(restored.current_generation(), Some(41));
    assert!(matches!(
        restored.status(41, &profile),
        ClientCapabilityStatus::Supported { .. }
    ));
}

fn profile(id: &str, codec: &str, dimensions: (u32, u32)) -> ClientCapabilityProfile {
    ClientCapabilityProfile::try_new(id, Some(codec), Some(dimensions))
        .expect("valid test fixture")
        .with_persistent_identity(true)
}
