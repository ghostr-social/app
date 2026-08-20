use super::support::{chunk_request, range_retrieval};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};

#[test]
fn same_range_from_a_replacement_representation_cancels_the_old_attempt() {
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta("https://a.example/video"));
    let old_identity = catalog
        .transfer_identity(&post, "https://a.example/video")
        .unwrap();
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 8),
    };
    let mut active = InFlightChunks::new();
    let old = active.next_attempt(chunk.clone(), old_identity);
    let (handle, token) = cancel_pair();
    active.insert(
        &old,
        chunk_request(chunk.clone(), PreemptionAuthority::Transition),
        "a.example".to_owned(),
        0,
        handle,
    );

    catalog.upsert(post.clone(), meta("https://b.example/video"));
    let replacement = transfer(&catalog, chunk, "https://b.example/video");
    active.reconcile(&[replacement], 1);

    assert!(token.is_cancelled());
    assert_eq!(active.finish(&old), CompletionStatus::Cancelled);
}

#[test]
fn binding_change_cancels_obsolete_committed_work_before_replanning() {
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta("https://a.example/video"));
    let old_identity = catalog
        .transfer_identity(&post, "https://a.example/video")
        .unwrap();
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 8),
    };
    let mut active = InFlightChunks::new();
    let old = active.next_attempt(chunk.clone(), old_identity);
    let (handle, token) = cancel_pair();
    active.insert(
        &old,
        chunk_request(chunk, PreemptionAuthority::Transition),
        "a.example".to_owned(),
        5_000,
        handle,
    );

    let replacement = catalog.upsert(post, meta("https://b.example/video"));
    active.cancel_obsolete(&replacement);

    assert!(token.is_cancelled());
    assert_eq!(active.len(), 1, "obsolete work remains fenced until ack");
    assert_eq!(active.finish(&old), CompletionStatus::Cancelled);
    assert_eq!(active.len(), 0);
}

fn transfer(catalog: &Catalog, chunk: ChunkId, url: &str) -> PlannedTransfer {
    let retrieval = range_retrieval(chunk.range);
    PlannedTransfer {
        identity: catalog.transfer_identity(&chunk.post, url).unwrap(),
        request: RangeRequest {
            chunk,
            authority: PreemptionAuthority::Transition,
            score: 1.0,
            contiguous_depth_bytes: 0,
        },
        url: url.to_owned(),
        retrieval,
        commitment_until_ms: 0,
    }
}

fn meta(url: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![url.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
