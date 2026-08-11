use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::{cancel_pair, CancelToken};
use crate::manager::inflight::InFlightChunks;
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn unplanned_same_identity_io_is_cancelled_even_with_spare_capacity() {
    let mut active = InFlightChunks::new();
    let (_, token) = insert(&mut active, transfer("far", 0, Tier::T4Speculative));
    let local = transfer("local", 0, Tier::T2Startability);

    active.reconcile(&[local], 2);

    assert!(token.is_cancelled());
}

#[test]
fn disjoint_urgent_work_cancels_same_post_speculation() {
    let mut active = InFlightChunks::new();
    let (_, token) = insert(&mut active, transfer("current", 8, Tier::T4Speculative));
    let urgent = transfer("current", 0, Tier::T0PlaybackEmergency);

    active.reconcile(&[urgent], 1);

    assert!(token.is_cancelled());
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
