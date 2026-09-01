use crate::client_capability::{
    CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal,
    ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus,
};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{HttpGenerationKey, HttpGenerationLease, HttpGenerationStamp};
use ghostr_engine::PostId;

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn local_verification_during_an_attempt_waits_for_a_verified_attempt() {
    let mutable = mutable_profile("v1");
    let verified = profile(true);
    let attempt = CapabilityAttempt::new(3, 5);
    let mut model = ClientCapabilityModel::default();

    observe(
        &mut model,
        mutable.clone(),
        attempt,
        CapabilitySignal::Initializing,
    );
    observe(
        &mut model,
        verified.clone(),
        attempt,
        CapabilitySignal::UnsupportedFailure,
    );

    assert_eq!(model.status(11, &verified), ClientCapabilityStatus::Unknown);
    assert_eq!(model.status(11, &mutable), ClientCapabilityStatus::Unknown);
    let state = serde_json::to_value(model.state()).expect("valid test fixture");
    assert_eq!(
        state["records"]
            .as_array()
            .expect("valid test fixture")
            .len(),
        0
    );
}

#[test]
fn validator_change_during_an_attempt_cannot_relabel_decoder_result() {
    let first = mutable_profile("v1");
    let changed = mutable_profile("v2");
    let attempt = CapabilityAttempt::new(3, 6);
    let mut model = ClientCapabilityModel::default();

    observe(
        &mut model,
        first.clone(),
        attempt,
        CapabilitySignal::Initializing,
    );
    observe(
        &mut model,
        changed.clone(),
        attempt,
        CapabilitySignal::UnsupportedFailure,
    );

    assert_eq!(model.status(11, &first), ClientCapabilityStatus::Unknown);
    assert_eq!(model.status(11, &changed), ClientCapabilityStatus::Unknown);
}

fn profile(persistent: bool) -> ClientCapabilityProfile {
    ClientCapabilityProfile::try_new("representation", Some("avc1"), Some((720, 1_280)))
        .expect("valid test fixture")
        .with_persistent_identity(persistent)
}

fn mutable_profile(validator: &str) -> ClientCapabilityProfile {
    let validator =
        EvidenceValidator::strong_etag(format!("\"{validator}\"")).expect("valid test fixture");
    let key = HttpGenerationKey::try_new(SOURCE, Some(validator)).expect("valid test fixture");
    let generation = HttpGenerationStamp::from_trusted(
        HttpGenerationLease::try_new(key, 1).expect("valid test fixture"),
    );
    profile(false).with_volatile_authority(&PostId::new("post"), SOURCE, Some(generation))
}

fn observe(
    model: &mut ClientCapabilityModel,
    profile: ClientCapabilityProfile,
    attempt: CapabilityAttempt,
    signal: CapabilitySignal,
) {
    model.observe(CapabilityObservation::new(
        11,
        attempt,
        profile,
        CapabilityEvent::new(20, signal),
    ));
}
