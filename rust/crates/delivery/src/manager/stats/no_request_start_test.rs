use super::StatsKeeper;
use crate::manager::inflight::InFlightChunks;
use crate::manager::transfers::{ChunkDone, ProbeObservation};
use core::time::Duration;
use ghostr_engine::{ByteRange, ChunkId, PostId};

#[tokio::test]
async fn failures_before_request_start_do_not_train_remote_performance() {
    let root = crate::tests::support::temp_directory("ghostr-no-request-stats");
    let path = root.join("host_stats.json");
    let mut keeper = StatsKeeper::load(path.clone(), Duration::ZERO).await;
    note_unstarted_chunk(&mut keeper);
    keeper.note_probe(&ProbeObservation {
        post: PostId::new("probe"),
        url: "https://probe.example/video.mp4".to_owned(),
        outcome: Err(anyhow::anyhow!("request construction failed")),
        attempt_context: None,
    });

    assert_eq!(keeper.stats().failure_ratio("chunk.example"), 0.0);
    assert_eq!(keeper.stats().failure_ratio("probe.example"), 0.0);
    keeper.save_now().await;
    assert!(
        !path.exists(),
        "no remote attempt means no persisted evidence"
    );
    std::fs::remove_dir_all(root).expect("remove fixture");
}

fn note_unstarted_chunk(keeper: &mut StatsKeeper) {
    let post = PostId::new("chunk");
    let url = "https://chunk.example/video.mp4";
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 1),
    };
    let mut inflight = InFlightChunks::new();
    let attempt =
        inflight.next_attempt(chunk, crate::tests::support::transfer_identity(&post, url));
    keeper.note_chunk(&ChunkDone {
        attempt,
        url: url.to_owned(),
        outcome: Err(anyhow::anyhow!("request construction failed")),
        received_bytes: 0,
        origin: None,
        open_body: None,
        request_started: false,
        whole_body_completion: None,
        response_evidence: None,
    });
}
