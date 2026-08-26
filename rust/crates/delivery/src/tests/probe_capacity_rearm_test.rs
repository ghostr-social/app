use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::adaptive::ProbeClaimRefusal;
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn full_probe_pool_does_not_consume_invalidated_head_history() {
    let active = PostId::new("active");
    let stale = PostId::new("stale");
    let mut catalog = Catalog::new();
    let active_binding = catalog.upsert(active.clone(), metadata("active"));
    let stale_binding = catalog.upsert(stale.clone(), metadata("stale"));
    let active_identity = active_binding.transfer(source("active")).expect("valid test fixture");
    let stale_identity = stale_binding.transfer(source("stale")).expect("valid test fixture");
    assert!(catalog.learn_head_observation_for(
        &stale_identity,
        observation(Some(16), Some(true), "v1", 1)
    ));
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    probes
        .claim_selected(query(&catalog, &retry, &active, source("active")))
        .expect("valid test fixture");
    probes.learned(
        &stale_identity,
        catalog.http_generation_for(&stale_identity),
    );
    assert!(catalog.learn_response_observation_for(
        &stale_identity,
        observation(None, None, "v2", 2)
    ));

    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &stale, source("stale"))),
        Err(ProbeClaimRefusal::PoolAtCapacity)
    );
    assert!(probes.has_completed_identity(&stale_identity));
    assert!(!probes.has_completed_identity(&active_identity));

    probes.release(&active);
    assert!(probes
        .claim_selected(query(&catalog, &retry, &stale, source("stale")))
        .is_ok());
}

fn query<'a>(
    catalog: &'a Catalog,
    retry: &'a RetryBook,
    post: &'a PostId,
    source: &'a str,
) -> ProbeClaimQuery<'a> {
    ProbeClaimQuery {
        catalog,
        retry,
        post,
        source,
        observed_at_ms: 2,
    }
}

fn observation(
    size: Option<u64>,
    ranges: Option<bool>,
    etag: &str,
    at: u64,
) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: size,
            accept_ranges: ranges,
            host: None,
        },
        None,
        at,
        EvidenceValidator::strong_etag(format!("\"{etag}\"")),
    )
}

fn metadata(id: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![source(id).to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}

fn source(id: &str) -> &str {
    match id {
        "active" => "https://media.example/active.mp4",
        _ => "https://media.example/stale.mp4",
    }
}
