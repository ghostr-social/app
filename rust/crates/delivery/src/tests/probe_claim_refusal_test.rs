use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::adaptive::{DecisionOutcome, ProbeClaimRefusal};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn selected_probe_refusals_preserve_the_specific_pool_state() {
    let first = PostId::new("first");
    let second = PostId::new("second");
    let mut catalog = Catalog::new();
    catalog.upsert(first.clone(), meta("first"));
    catalog.upsert(second.clone(), meta("second"));
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);
    let first_url = meta("first").urls[0].clone();
    let second_url = meta("second").urls[0].clone();

    let identity = probes
        .claim_selected(query(&catalog, &retry, &first, &first_url))
        .expect("valid test fixture");
    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &first, &first_url)),
        Err(ProbeClaimRefusal::AlreadyProbing)
    );
    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &second, &second_url)),
        Err(ProbeClaimRefusal::PoolAtCapacity)
    );
    assert_eq!(
        serde_json::to_value(DecisionOutcome::ClaimRefused {
            reason: ProbeClaimRefusal::PoolAtCapacity,
        })
        .expect("valid test fixture"),
        serde_json::json!({"status": "claim_refused", "reason": "pool_at_capacity"})
    );

    probes.learned(&identity, None);
    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &first, &first_url)),
        Err(ProbeClaimRefusal::AlreadyProbed)
    );
}

#[test]
fn deferred_and_cooling_probes_have_distinct_refusals() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let metadata = meta("post");
    let source = metadata.urls[0].clone();
    catalog.upsert(post.clone(), metadata);
    let mut retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);

    let identity = probes
        .claim_selected(query(&catalog, &retry, &post, &source))
        .expect("valid test fixture");
    probes.defer_to_body(&identity);
    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &post, &source)),
        Err(ProbeClaimRefusal::DeferredToBody)
    );
    probes.body_finished(&identity);
    retry.cool_down(post.clone()).expect("valid test fixture");
    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &post, &source)),
        Err(ProbeClaimRefusal::RetryCooling)
    );
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
        observed_at_ms: 0,
    }
}

fn meta(id: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
