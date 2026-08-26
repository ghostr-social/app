use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::adaptive::ProbeClaimRefusal;
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";
const FIRST: &str = "https://cdn-a.example/video.mp4";
const SECOND: &str = "https://cdn-b.example/video.mp4";

#[test]
fn validatorless_response_cannot_rearm_unbound_probe_history() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let identity = catalog.upsert(post.clone(), metadata()).transfer(SOURCE).expect("valid test fixture");
    let stamp = catalog
        .learn_head_observation_with_stamp_for(&identity, observation(FIRST, 1))
        .expect("valid test fixture");
    let mut probes = MetadataProbePool::new(1);
    probes.learned_probe(&identity, stamp, false);
    assert!(!catalog.learn_response_observation_for(&identity, observation(SECOND, 2)));

    let retry = RetryBook::new(RetryPolicy::default());
    assert!(matches!(
        probes.claim_selected(ProbeClaimQuery {
            catalog: &catalog,
            retry: &retry,
            post: &post,
            source: SOURCE,
            observed_at_ms: 2,
        }),
        Err(ProbeClaimRefusal::AlreadyProbed)
    ));
}

fn observation(final_url: &str, at: u64) -> HttpObservation {
    HttpObservation::new(LearnedFacts::default(), None, at, None)
        .with_final_url(final_url)
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
