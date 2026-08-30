use super::support::{chunk_request, range_profile, range_retrieval, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[test]
fn a_cancelled_attempt_retains_identity_until_its_terminal_ack() {
    let mut active = InFlightChunks::new();
    let chunk = ChunkId {
        post: PostId::new("old"),
        range: ByteRange::new(0, 8),
    };
    let attempt = active.next_attempt(
        chunk.clone(),
        transfer_identity(&chunk.post, "https://slow.example/video"),
    );
    let (handle, _token) = cancel_pair();
    active.insert(
        &attempt,
        chunk_request(chunk.clone(), PreemptionAuthority::Speculative),
        "slow.example".to_owned(),
        0,
        handle,
    );

    assert!(active.can_cancel_action(attempt.id()));
    assert!(active.cancel_action(attempt.id()));
    assert!(!active.can_cancel_action(attempt.id()));
    assert!(!active.cancel_action(attempt.id()));
    attempt.mark_io_finished();
    let terminal = active.actions();
    assert!(terminal[0].cancelling());
    assert!(terminal[0].io_finished());

    assert_eq!(active.finish(&attempt), CompletionStatus::Cancelled);
}

#[test]
fn a_terminal_attempt_rejects_a_cancel_from_a_stale_planner_snapshot() {
    let mut active = InFlightChunks::new();
    let transfer = planned_transfer();
    let attempt = active.next_attempt(transfer.request.chunk.clone(), transfer.identity.clone());
    let (handle, _token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request.clone(),
        "slow.example".to_owned(),
        0,
        handle,
    );

    assert!(!active.actions()[0].io_finished());
    attempt.mark_io_finished();

    assert!(!active.can_cancel_action(attempt.id()));
    assert!(!active.cancel_action(attempt.id()));
    assert_eq!(active.finish(&attempt), CompletionStatus::Current);
}

#[test]
fn a_cancelling_attempt_fences_an_identical_origin_relaunch() {
    let mut active = InFlightChunks::new();
    let transfer = planned_transfer();
    let attempt = active.next_attempt(transfer.request.chunk.clone(), transfer.identity.clone());
    let (handle, _token) = cancel_pair();
    active.insert(
        &attempt,
        transfer.request.clone(),
        "slow.example".to_owned(),
        0,
        handle,
    );

    assert!(active.cancel_action(attempt.id()));
    assert!(active.contains_transfer(&transfer));
    assert_eq!(active.finish(&attempt), CompletionStatus::Cancelled);
    assert!(!active.contains_transfer(&transfer));
}

fn planned_transfer() -> PlannedTransfer {
    let post = PostId::new("reserve");
    let url = "https://slow.example/video".to_owned();
    let bytes = ByteRange::new(0, 65_536);
    PlannedTransfer {
        control_mode: ghostr_engine::adaptive::ControlMode::Normal,
        identity: transfer_identity(&post, &url),
        request: chunk_request(
            ChunkId { post, range: bytes },
            PreemptionAuthority::Transition,
        ),
        url,
        retrieval: range_retrieval(bytes),
        profile: range_profile(bytes.len()),
        commitment_until_ms: 0,
    }
}
