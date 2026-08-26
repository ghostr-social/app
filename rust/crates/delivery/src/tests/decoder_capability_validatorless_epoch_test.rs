
use crate::client_capability::{CapabilityAttempt, CapabilityEvent, CapabilityObservation, CapabilitySignal, ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus};
use ghostr_engine::representation::{
    HttpGenerationKey, HttpGenerationLease, HttpGenerationStamp,
};
use ghostr_engine::PostId;

const SOURCE: &str = "https://media.example/validatorless.mp4";

#[test]
fn validatorless_decoder_rejection_expires_with_its_http_epoch() {
    let first = profile(1);
    let replacement = profile(2);
    let attempt = CapabilityAttempt::new(9, 1);
    let mut model = ClientCapabilityModel::default();

    observe(
        &mut model,
        first.clone(),
        attempt,
        CapabilitySignal::Initializing,
    );
    observe(
        &mut model,
        first.clone(),
        attempt,
        CapabilitySignal::UnsupportedFailure,
    );

    assert_eq!(model.status(7, &first), ClientCapabilityStatus::Unsupported);
    assert_eq!(model.status(7, &replacement), ClientCapabilityStatus::Unknown);
}

fn profile(epoch: u64) -> ClientCapabilityProfile {
    let key = HttpGenerationKey::try_new(SOURCE, None).expect("valid test fixture");
    let lease = HttpGenerationLease::try_new(key, epoch).expect("valid test fixture");
    let stamp = HttpGenerationStamp::from_trusted(lease);
    ClientCapabilityProfile::try_new("representation", Some("avc1"), Some((720, 1_280)))
        .expect("valid test fixture")
        .with_volatile_authority(&PostId::new("post"), SOURCE, Some(stamp))
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
        CapabilityEvent::new(20, signal),
    ));
}
