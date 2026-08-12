use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn missing_urgent_work_preempts_only_the_lowest_priority_transfer() {
    let mut active = InFlightChunks::new();
    let (protected, protected_token) =
        insert(&mut active, transfer("protected", 0, Tier::T2Startability));
    let (far, far_token) = insert(&mut active, transfer("far", 0, Tier::T4Speculative));
    let urgent = transfer("urgent", 0, Tier::T2Startability);
    let planned = [protected.clone(), far, urgent];

    active.reconcile(&planned, 2);

    assert!(!protected_token.is_cancelled());
    assert!(far_token.is_cancelled());
}

#[test]
fn equal_priority_victim_selection_uses_range_order() {
    let mut active = InFlightChunks::new();
    let (earlier, earlier_token) = insert(&mut active, transfer("far", 0, Tier::T4Speculative));
    let (later, later_token) = insert(&mut active, transfer("far", 8, Tier::T4Speculative));
    let urgent = transfer("urgent", 0, Tier::T2Startability);
    let planned = [earlier, later, urgent];

    active.reconcile(&planned, 2);

    assert!(!earlier_token.is_cancelled());
    assert!(later_token.is_cancelled());
}

fn insert(
    active: &mut InFlightChunks,
    transfer: PlannedTransfer,
) -> (PlannedTransfer, CancelToken) {
    let attempt = active.next_attempt(transfer.request.chunk.clone(), transfer.identity.clone());
    let (handle, token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request.clone(),
        transfer.request.chunk.post.as_str().to_owned(),
        handle,
    );
    (transfer, token)
}

fn transfer(post: &str, start: u64, tier: Tier) -> PlannedTransfer {
    let post = PostId::new(post);
    let url = format!("https://{}.example/video.mp4", post.as_str());
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: chunk_request(
            ChunkId {
                post,
                range: ByteRange::new(start, start + 8),
            },
            tier,
        ),
        url,
    }
}
