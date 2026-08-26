use super::super::{HedgeCase, ALTERNATE, PRIMARY};
use crate::manager::state::DeliveryState;
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::PostId;

pub(in crate::tests::warp_hedge_plan_fixture) fn learn_identity(
    state: &mut DeliveryState,
    post: &PostId,
    case: HedgeCase,
) {
    learn_generation(state, post, PRIMARY, "\"primary-v1\"", 1);
    learn_generation(state, post, ALTERNATE, "\"alternate-v1\"", 1);
    match case {
        HedgeCase::AdvertisedOnly => {}
        HedgeCase::PrimaryVerifiedOnly => verify(state, post, PRIMARY),
        _ => {
            verify(state, post, PRIMARY);
            verify(state, post, ALTERNATE);
        }
    }
    if matches!(case, HedgeCase::AlternateRotated) {
        learn_generation(state, post, ALTERNATE, "\"alternate-v2\"", 3);
    }
}

fn learn_generation(
    state: &mut DeliveryState,
    post: &PostId,
    source: &str,
    etag: &str,
    observed_at_ms: u64,
) {
    let identity = state.catalog().transfer_identity(post, source).expect("valid test fixture");
    let observation = HttpObservation::new(
        LearnedFacts {
            content_length: Some(1_000_000),
            accept_ranges: Some(true),
            host: None,
        },
        None,
        observed_at_ms,
        EvidenceValidator::strong_etag(etag),
    )
    .with_final_url(source);
    assert!(state
        .catalog_mut()
        .learn_response_observation_for(&identity, observation));
}

fn verify(state: &mut DeliveryState, post: &PostId, source: &str) {
    let identity = state.catalog().transfer_identity(post, source).expect("valid test fixture");
    let generation = state.catalog().http_generation_for(&identity).expect("valid test fixture");
    assert!(state.catalog_mut().record_verified_hash_for_generation(
        &identity,
        &"11".repeat(32),
        source,
        2,
        &generation,
    ));
}
