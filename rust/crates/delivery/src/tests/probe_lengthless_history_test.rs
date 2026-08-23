use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::adaptive::ProbeClaimRefusal;
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn successful_lengthless_head_does_not_loop_on_an_older_stale_size() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let identity = catalog.upsert(post.clone(), metadata()).transfer(SOURCE).unwrap();
    assert!(catalog.learn_head_observation_for(&identity, observation(Some(16), 1)));
    let stamp = catalog
        .learn_head_observation_with_stamp_for(&identity, observation(None, 2))
        .expect("accepted lengthless HEAD");
    let mut probes = MetadataProbePool::new(1);
    probes.learned_probe(&identity, stamp, false);
    let retry = RetryBook::new(RetryPolicy::default());

    assert_eq!(
        probes.claim_selected(ProbeClaimQuery {
            catalog: &catalog,
            retry: &retry,
            post: &post,
            source: SOURCE,
            observed_at_ms: DAY_MS,
        }),
        Err(ProbeClaimRefusal::AlreadyProbed),
    );
}

fn observation(size: Option<u64>, order: u64) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: size,
            ..LearnedFacts::default()
        },
        None,
        ghostr_engine::evidence::EvidenceTime::ordered(1_000, order),
        None,
    )
    .with_final_url(SOURCE)
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
