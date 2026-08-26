use super::support::{range_retrieval, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn higher_value_transition_preempts_an_unretained_peer() {
    let seed = transfer("near", 0, 0, 1.0);
    let shifted = transfer("near", 16, 16, 1.0);
    let peer = transfer("far", 0, 0, 2.0);
    let mut active = InFlightChunks::new();
    let attempt = active.next_attempt(seed.request.chunk.clone(), seed.identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        seed.request,
        "media.example".to_owned(),
        seed.commitment_until_ms,
        handle,
    );

    active.reconcile(&[shifted, peer], 1);

    assert!(token.is_cancelled());
}

fn transfer(post: &str, start: u64, depth: u64, score: f64) -> PlannedTransfer {
    let post = PostId::new(post);
    let url = format!("https://media.example/{}.mp4", post.as_str());
    PlannedTransfer {
        control_mode: ghostr_engine::adaptive::ControlMode::Normal,
        identity: transfer_identity(&post, &url),
        request: RangeRequest {
            chunk: ChunkId {
                post,
                range: ByteRange::new(start, start + 64),
            },
            authority: PreemptionAuthority::Transition,
            score,
            contiguous_depth_bytes: depth,
        },
        url,
        retrieval: range_retrieval(ByteRange::new(start, start + 64)),
        commitment_until_ms: 0,
    }
}
