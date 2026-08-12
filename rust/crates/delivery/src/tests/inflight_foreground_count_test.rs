use super::support::planned_transfer;
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::ChunkId;

#[test]
fn foreground_count_excludes_the_active_protected_seed() {
    let mut active = InFlightChunks::new();
    let emergency = insert(
        &mut active,
        "emergency",
        PreemptionAuthority::PlaybackCritical,
    );
    insert(&mut active, "tail", PreemptionAuthority::PlaybackCritical);
    insert(&mut active, "seed", PreemptionAuthority::Transition);

    assert_eq!(active.foreground_len(), 2);
    assert!(active.cancel(&emergency));
    assert_eq!(active.foreground_len(), 1);
}

fn insert(active: &mut InFlightChunks, name: &str, authority: PreemptionAuthority) -> ChunkId {
    let transfer = planned_transfer(name, "same.example", authority);
    let chunk = transfer.request.chunk.clone();
    let attempt = active.next_attempt(chunk.clone(), transfer.identity);
    let (handle, _token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request,
        "same.example".to_owned(),
        transfer.commitment_until_ms,
        handle,
    );
    chunk
}
