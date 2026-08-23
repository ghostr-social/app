use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::adaptive::ProbeClaimRefusal;
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";
const OBSERVED_AT_MS: u64 = 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn probe_claim_uses_fused_range_evidence_at_the_planning_time() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), metadata());
    let identity = binding.transfer(SOURCE).expect("source identity");
    let response = HttpObservation::new(
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
        None,
        OBSERVED_AT_MS,
        EvidenceValidator::strong_etag("\"generation-1\""),
    );
    assert!(catalog.learn_response_observation_for(&identity, response));
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);

    let fresh = query(&catalog, &retry, &post, OBSERVED_AT_MS);
    assert_eq!(
        probes.claim_selected(fresh),
        Err(ProbeClaimRefusal::EvidenceComplete)
    );
    probes.learned(&identity, catalog.http_generation_for(&identity));
    let stale = query(&catalog, &retry, &post, OBSERVED_AT_MS + DAY_MS);
    assert_eq!(
        probes.claim_selected(stale),
        Err(ProbeClaimRefusal::AlreadyProbed)
    );
}

fn query<'a>(
    catalog: &'a Catalog,
    retry: &'a RetryBook,
    post: &'a PostId,
    observed_at_ms: u64,
) -> ProbeClaimQuery<'a> {
    ProbeClaimQuery {
        catalog,
        retry,
        post,
        source: SOURCE,
        observed_at_ms,
    }
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
