use super::support::{chunk_request, transfer_identity};
use crate::chunk::cancel::cancel_pair;
use crate::manager::inflight::{ChunkAttempt, InFlightChunks};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ByteRange, ChunkId, PostId};

// TODO: Cover prepared store/admission cleanup through the full manager grant boundary.

#[test]
fn terminal_primary_rejects_stale_prelaunch_hedge_authorization() {
    let mut active = InFlightChunks::new();
    let post = PostId::new("current");
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 65_536),
    };
    let primary = attempt(&mut active, &chunk, "https://slow.example/video.mp4");
    insert(&mut active, &primary);

    assert!(!active.actions()[0].io_finished());
    primary.mark_io_finished();
    assert!(!active.authorize_hedge(primary.id()));

    let alternate = attempt(&mut active, &chunk, "https://fast.example/video.mp4");
    insert(&mut active, &alternate);
    assert!(!active.link_hedge(primary.id(), alternate.id()));
}

#[test]
fn hedge_authorized_while_live_may_link_after_primary_io_finishes() {
    let mut active = InFlightChunks::new();
    let post = PostId::new("current");
    let chunk = ChunkId {
        post,
        range: ByteRange::new(0, 65_536),
    };
    let primary = attempt(&mut active, &chunk, "https://slow.example/video.mp4");
    insert(&mut active, &primary);

    assert!(active.authorize_hedge(primary.id()));
    primary.mark_io_finished();
    let alternate = attempt(&mut active, &chunk, "https://fast.example/video.mp4");
    insert(&mut active, &alternate);

    assert!(active.actions()[0].io_finished());
    assert!(active.link_hedge(primary.id(), alternate.id()));
}

fn attempt(
    active: &mut InFlightChunks,
    chunk: &ChunkId,
    source: &str,
) -> ChunkAttempt {
    active.next_attempt(
        chunk.clone(),
        transfer_identity(&chunk.post, source),
    )
}

fn insert(active: &mut InFlightChunks, attempt: &ChunkAttempt) {
    let (handle, _token) = cancel_pair();
    active.insert(
        attempt,
        chunk_request(
            attempt.chunk.clone(),
            PreemptionAuthority::PlaybackCritical,
        ),
        "example".to_owned(),
        0,
        handle,
    );
}
