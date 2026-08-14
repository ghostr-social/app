use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use crate::tests::support::transfer_identity;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::HashSet;

#[test]
fn finished_body_remains_present_until_its_completion_is_absorbed() {
    let post = PostId::new("pending-completion");
    let url = "https://media.example/video.mp4";
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 64),
    };
    let request = RangeRequest {
        chunk: chunk.clone(),
        authority: PreemptionAuthority::Speculative,
        score: 1.0,
        contiguous_depth_bytes: 0,
    };
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk, transfer_identity(&post, url));
    let (handle, _) = cancel_pair();
    active.insert(&attempt, request, "media.example".to_owned(), 0, handle);
    attempt.mark_io_finished();

    assert_eq!(active.body_posts(), HashSet::from([post]));
    active.finish(&attempt);
    assert!(active.body_posts().is_empty());
}
