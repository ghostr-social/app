use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{
    ActionRegistration, InFlightChunks, ResponseGenerationFence,
};
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn focus_cancellation_preserves_the_admitted_response_generation_fence() {
    let post = PostId::new("post");
    let identity = transfer_identity(&post, "https://media.example/video.mp4");
    let chunk = ChunkId {
        post,
        range: ByteRange::new(0, 8),
    };
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(chunk.clone(), identity);
    let (handle, _token) = cancel_pair();
    active.insert_action(ActionRegistration {
        attempt: &attempt,
        priority: chunk_request(chunk.clone(), PreemptionAuthority::Transition),
        retrieval: RetrievalRequest::FetchRange {
            bytes: chunk.range,
            promotion: None,
        },
        host: "media.example".into(),
        committed_until_ms: 0,
        launched_at_ms: 0,
        handle,
        store_action: None,
        committed_network_bytes: None,
        exploration_claim: None,
    });
    assert!(active.adopt_action_scoped_generation(&attempt, None));

    assert!(active.cancel(&chunk));

    assert_eq!(
        active.policy_limit_generation(&attempt),
        Some(ResponseGenerationFence::ActionScoped(None))
    );
}
