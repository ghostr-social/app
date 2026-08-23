use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";
const OBSERVED_AT_MS: u64 = 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn completed_head_history_rearms_when_its_size_is_stale() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), metadata());
    let identity = binding.transfer(SOURCE).unwrap();
    let head = HttpObservation::new(
        LearnedFacts {
            content_length: Some(16),
            ..LearnedFacts::default()
        },
        None,
        OBSERVED_AT_MS,
        None,
    );
    let stamp = catalog
        .learn_head_observation_with_stamp_for(&identity, head)
        .unwrap();
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    probes.learned_probe(&identity, stamp, true);

    let query = ProbeClaimQuery {
        catalog: &catalog,
        retry: &retry,
        post: &post,
        source: SOURCE,
        observed_at_ms: OBSERVED_AT_MS + DAY_MS,
    };
    assert!(probes.claim_selected(query).is_ok());
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
