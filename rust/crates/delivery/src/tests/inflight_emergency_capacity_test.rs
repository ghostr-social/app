use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn disjoint_current_demand_fully_preempts_lower_priority_work() {
    let current = PostId::new("current");
    let current_head = chunk("current", 0);
    let demanded = chunk("current", 32);
    let ahead = [
        chunk("ahead-a", 0),
        chunk("ahead-b", 8),
        chunk("ahead-c", 16),
    ];
    let mut active = InFlightChunks::new();
    let current_token = insert(&mut active, &current_head);
    let mut ahead_tokens = Vec::new();
    for chunk in &ahead {
        ahead_tokens.push(insert(&mut active, chunk));
    }
    let priority = [
        demanded.clone(),
        current_head.clone(),
        ahead[0].clone(),
        ahead[1].clone(),
        ahead[2].clone(),
    ];

    active.preempt_for_current(&current, &priority, 2);

    assert_eq!(active.len(), 1, "one slot must be free for demand");
    assert!(active.contains(&current_head));
    assert!(!current_token.is_cancelled());
    assert!(ahead_tokens.iter().all(|token| token.is_cancelled()));
    let demand_token = insert(&mut active, &demanded);
    assert_eq!(active.len(), 2, "demand is admitted at capacity");
    assert!(active.contains(&demanded));
    assert!(!demand_token.is_cancelled());
}

#[test]
fn satisfied_current_demand_does_not_preempt_work() {
    let current = PostId::new("current");
    let demanded = chunk("current", 0);
    let mut active = InFlightChunks::new();
    let token = insert(&mut active, &demanded);

    active.preempt_for_current(&current, std::slice::from_ref(&demanded), 1);

    assert!(!token.is_cancelled());
}

#[test]
fn current_demand_without_a_lower_priority_candidate_does_not_preempt() {
    let current = PostId::new("current");
    let demanded = chunk("current", 0);
    let ahead = chunk("ahead", 0);
    let mut active = InFlightChunks::new();
    let token = insert(&mut active, &ahead);

    active.preempt_for_current(&current, &[demanded], 1);

    assert!(!token.is_cancelled());
}

fn chunk(post: &str, start: u64) -> ChunkId {
    ChunkId {
        post: PostId::new(post),
        range: ByteRange::new(start, start + 8),
    }
}

fn insert(active: &mut InFlightChunks, chunk: &ChunkId) -> crate::chunk::cancel::CancelToken {
    let identity = transfer_identity(&chunk.post, "https://slow.example/video");
    let attempt = active.next_attempt(chunk.clone(), identity);
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk.clone(), PreemptionAuthority::Speculative),
        "slow.example".to_owned(),
        0,
        handle,
    );
    token
}
