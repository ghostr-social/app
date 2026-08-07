use super::support::temp_directory;
use crate::engine::{ByteRange, ChunkId, PostId};
use crate::video::chunk_downloader::ChunkResult;
use crate::video::delivery_inflight::InFlightChunks;
use crate::video::delivery_stats::StatsKeeper;
use crate::video::delivery_transfers::{ChunkDone, ProbeDone};
use crate::video::media_probe::ProbeResult;
use std::time::Duration;

#[tokio::test]
async fn unhosted_outcomes_do_not_dirty_or_persist_host_stats() {
    let root = temp_directory("ghostr-unhosted-stats");
    let path = root.join("host_stats.json");
    let mut keeper = StatsKeeper::load(path.clone(), Duration::ZERO).await;
    let post = PostId::new("clip");
    let mut inflight = InFlightChunks::new();
    let attempt = inflight.next_attempt(ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 1),
    });
    keeper.note_chunk(&ChunkDone {
        attempt,
        url: "not a URL".to_owned(),
        elapsed: Duration::from_millis(1),
        outcome: Ok(ChunkResult {
            bytes_written: 1,
            accept_ranges: true,
            cancelled: false,
            total_bytes: Some(1),
            request_started: true,
        }),
    });
    keeper.note_probe(&ProbeDone {
        post,
        url: "still not a URL".to_owned(),
        outcome: Ok(ProbeResult {
            content_length: Some(1),
            accept_ranges: true,
            content_type: Some("video/mp4".to_owned()),
            ttfb: Duration::from_millis(1),
        }),
    });

    keeper.save_now().await;

    assert!(!path.exists());
    std::fs::remove_dir_all(root).expect("remove test directory");
}
