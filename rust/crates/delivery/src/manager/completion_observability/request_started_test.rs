use super::transfer_event;
use crate::manager::inflight::{ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::manager::transfers::ChunkDone;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};

#[test]
fn a_transport_failure_after_send_still_counts_the_request() {
    let (attempt, url) = failed_attempt();
    let done = ChunkDone {
        attempt,
        url,
        outcome: Err(anyhow::anyhow!("response headers timed out")),
        received_bytes: 0,
        origin: None,
        request_started: true,
        whole_body_completion: None,
        response_evidence: None,
    };

    let event = transfer_event(&done, CompletionStatus::Current, None);

    assert!(event.request_started);
}

fn failed_attempt() -> (ChunkAttempt, String) {
    let post = PostId::new("failed");
    let url = "https://media.example/failed.mp4".to_owned();
    let meta = VideoMeta {
        urls: vec![url.clone()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1),
    };
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta);
    let identity = catalog.transfer_identity(&post, &url).expect("identity");
    let chunk = ChunkId {
        post,
        range: ByteRange::new(0, 4),
    };
    (InFlightChunks::new().next_attempt(chunk, identity), url)
}
