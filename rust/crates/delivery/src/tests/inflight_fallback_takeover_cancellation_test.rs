use super::support::chunk_request;
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};
use std::collections::HashMap;

const SOURCE: &str = "https://primary.example/video.mp4";

#[test]
fn exact_fallback_coverage_cancels_only_redundant_transport() {
    let post = PostId::new("post");
    let (catalog, identity) = catalog(&post);
    let mut active = InFlightChunks::new();
    let covered = insert(&mut active, &post, ByteRange::new(0, 32), &identity);
    let useful = insert(&mut active, &post, ByteRange::new(32, 64), &identity);
    let present = HashMap::from([(post, vec![ByteRange::new(16, 32), ByteRange::new(0, 16)])]);

    active.cancel_covered_without_body(&present, &HashMap::new(), &catalog);

    assert!(covered.is_cancelled());
    assert!(!useful.is_cancelled());
}

fn insert(
    active: &mut InFlightChunks,
    post: &PostId,
    range: ByteRange,
    identity: &TransferIdentity,
) -> CancelToken {
    let chunk = ChunkId {
        post: post.clone(),
        range,
    };
    let attempt = active.next_attempt(chunk.clone(), identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk, PreemptionAuthority::PlaybackCritical),
        "primary.example".into(),
        0,
        handle,
    );
    token
}

fn catalog(post: &PostId) -> (Catalog, TransferIdentity) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec![SOURCE.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(64),
            duration_ms: Some(1_000),
        },
    );
    let identity = binding.transfer(SOURCE).expect("current transfer");
    (catalog, identity)
}
