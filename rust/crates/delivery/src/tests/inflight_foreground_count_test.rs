use super::support::planned_transfer;
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::InFlightChunks;
use ghostr_engine::tiers::Tier;
use ghostr_engine::ChunkId;

#[test]
fn foreground_count_excludes_the_active_protected_seed() {
    let mut active = InFlightChunks::new();
    let emergency = insert(&mut active, "emergency", Tier::T0PlaybackEmergency);
    insert(&mut active, "tail", Tier::T1CurrentTail);
    insert(&mut active, "seed", Tier::T2Startability);

    assert_eq!(active.foreground_len(), 2);
    assert!(active.cancel(&emergency));
    assert_eq!(active.foreground_len(), 1);
}

fn insert(active: &mut InFlightChunks, name: &str, tier: Tier) -> ChunkId {
    let transfer = planned_transfer(name, "same.example", tier);
    let chunk = transfer.request.chunk.clone();
    let attempt = active.next_attempt(chunk.clone(), transfer.identity);
    let (handle, _token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request,
        "same.example".to_owned(),
        handle,
    );
    chunk
}
