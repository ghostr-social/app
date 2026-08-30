use super::support::chunk_request;
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::{ChunkAttempt, InFlightChunks};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease, TransferIdentity,
};
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://a.example/video";

#[test]
fn only_the_exact_http_generation_keeps_concurrent_work_alive() {
    let identity = identity();
    let chunk = ChunkId {
        post: identity.post().clone(),
        range: ByteRange::new(0, 8),
    };
    let mut active = InFlightChunks::new();
    let (first, first_token) = insert(&mut active, &identity, &chunk);
    let (_second, second_token) = insert(&mut active, &identity, &chunk);
    let original = generation(1);

    assert!(active.adopt_http_generation(&first, &original));
    assert!(!first_token.is_cancelled());
    assert!(second_token.is_cancelled());
    assert_eq!(active.http_generation(&first), Some(original));

    let (replacement, replacement_token) = insert(&mut active, &identity, &chunk);
    assert!(active.adopt_http_generation(&replacement, &generation(2)));
    assert!(
        first_token.is_cancelled(),
        "ABA epoch must fence the old response"
    );
    assert!(!replacement_token.is_cancelled());
}

#[test]
fn head_authority_defers_to_pending_body_headers_but_fences_open_stale_work() {
    let identity = identity();
    let chunk = ChunkId {
        post: identity.post().clone(),
        range: ByteRange::new(0, 8),
    };
    let mut active = InFlightChunks::new();
    let (pending, pending_token) = insert(&mut active, &identity, &chunk);
    let original = generation(1);

    active.enforce_http_authority(
        &identity,
        &HttpGenerationAuthority::Trusted(original.clone()),
    );
    assert!(!pending_token.is_cancelled());
    assert!(active.adopt_http_generation(&pending, &original));

    active.enforce_http_authority(&identity, &HttpGenerationAuthority::Trusted(generation(2)));
    assert!(pending_token.is_cancelled());
}

fn insert(
    active: &mut InFlightChunks,
    identity: &TransferIdentity,
    chunk: &ChunkId,
) -> (ChunkAttempt, CancelToken) {
    let attempt = active.next_attempt(chunk.clone(), identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk.clone(), PreemptionAuthority::Transition),
        "a.example".into(),
        0,
        handle,
    );
    (attempt, token)
}

fn identity() -> TransferIdentity {
    let mut catalog = Catalog::new();
    let post = PostId::new("post");
    catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec![SOURCE.into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(8),
            duration_ms: Some(1_000),
        },
    );
    catalog
        .transfer_identity(&post, SOURCE)
        .expect("valid test fixture")
}

fn generation(epoch: u64) -> HttpGenerationLease {
    let key = HttpGenerationKey::try_new(SOURCE, None).expect("valid test fixture");
    HttpGenerationLease::try_new(key, epoch).expect("valid test fixture")
}
