use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::evidence::EvidenceValidator;
use crate::{DeliveryKind, PostId, VideoMeta};

const PRIMARY: &str = "https://primary.example/video.mp4";
const ALTERNATE: &str = "https://alternate.example/video.mp4";
const DIGEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";

#[test]
fn hash_without_exact_response_generation_cannot_authorize_splicing() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), meta());
    let primary = binding.transfer(PRIMARY).expect("valid test fixture");
    let alternate = binding.transfer(ALTERNATE).expect("valid test fixture");
    learn(&mut catalog, &primary, "\"primary-v1\"", 1);
    learn(&mut catalog, &alternate, "\"alternate-v1\"", 1);

    assert!(catalog.record_verified_hash_for(&primary, DIGEST, PRIMARY, 2));
    assert!(catalog.record_verified_hash_for(&alternate, DIGEST, ALTERNATE, 2));
    assert_eq!(
        catalog.verified_mirror_digest(&post, PRIMARY, ALTERNATE),
        None
    );
}

#[test]
fn stale_completed_generation_cannot_authorize_the_current_mirror() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), meta());
    let primary = binding.transfer(PRIMARY).expect("valid test fixture");
    let alternate = binding.transfer(ALTERNATE).expect("valid test fixture");
    learn(&mut catalog, &primary, "\"primary-v1\"", 1);
    learn(&mut catalog, &alternate, "\"alternate-v1\"", 1);
    let primary_v1 = catalog
        .http_generation_for(&primary)
        .expect("valid test fixture");
    let alternate_v1 = catalog
        .http_generation_for(&alternate)
        .expect("valid test fixture");
    learn(&mut catalog, &primary, "\"primary-v2\"", 2);

    assert!(!catalog.record_verified_hash_for_generation(
        &primary,
        DIGEST,
        PRIMARY,
        3,
        &primary_v1,
    ));
    assert!(catalog.record_verified_hash_for_generation(
        &alternate,
        DIGEST,
        ALTERNATE,
        3,
        &alternate_v1,
    ));
    assert_eq!(
        catalog.verified_mirror_digest(&post, PRIMARY, ALTERNATE),
        None
    );
}

fn learn(
    catalog: &mut Catalog,
    identity: &crate::representation::TransferIdentity,
    etag: &str,
    observed_at_ms: u64,
) {
    let observation = HttpObservation::new(
        LearnedFacts::default(),
        None,
        observed_at_ms,
        EvidenceValidator::strong_etag(etag),
    )
    .with_final_url(identity.source().as_str());
    assert!(catalog.learn_response_observation_for(identity, observation));
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![PRIMARY.to_owned(), ALTERNATE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: Some(DIGEST.to_owned()),
        size_bytes: Some(1_000),
        duration_ms: Some(1_000),
    }
}
