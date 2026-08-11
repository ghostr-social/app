use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use crate::mutable_priority_queue::MutablePriorityQueue;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::scoring::ChunkRequest;
use ghostr_engine::tiers::Tier;
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
    active.insert(&old, "a.example".to_owned(), handle);

    catalog.upsert(post.clone(), meta("https://b.example/video"));
    let replacement = transfer(&catalog, chunk, "https://b.example/video");
    let mut queue = MutablePriorityQueue::new();
    queue.replace(vec![replacement]);
    active.cancel_absent(&queue.wanted());

    assert!(token.is_cancelled());
    assert_eq!(active.finish(&old), CompletionStatus::Superseded);
}

fn transfer(catalog: &Catalog, chunk: ChunkId, url: &str) -> PlannedTransfer {
    PlannedTransfer {
        identity: catalog.transfer_identity(&chunk.post, url).unwrap(),
        request: ChunkRequest {
            chunk,
            tier: Tier::T2Startability,
            score: 1.0,
        },
        url: url.to_owned(),
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
