use super::support::{temp_directory, transfer_identity};
use crate::chunk::downloader::ChunkResult;
use crate::manager::inflight::InFlightChunks;
use crate::manager::stats::StatsKeeper;
use crate::manager::transfers::{ChunkDone, ProbeObservation};
use crate::probe::media::ProbeResult;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::time::Duration;

#[tokio::test]
async fn unhosted_outcomes_do_not_dirty_or_persist_host_stats() {
    let root = temp_directory("ghostr-unhosted-stats");
    let path = root.join("host_stats.json");
    let mut keeper = StatsKeeper::load(path.clone(), Duration::ZERO).await;
    let post = PostId::new("clip");
    let mut inflight = InFlightChunks::new();
    let chunk = ChunkId {
        post: post.clone(),
        range: ByteRange::new(0, 1),
    };
    let attempt = inflight.next_attempt(chunk, transfer_identity(&post, "not a URL"));
    keeper.note_chunk(&ChunkDone {
        attempt,
        url: "not a URL".to_owned(),
        outcome: Ok(ChunkResult {
            bytes_written: 1,
            range_support: Some(true),
            range_ignored: false,
            cancelled: false,
            total_bytes: Some(1),
            promoted: false,
            request_started: true,
        }),
        received_bytes: 1,
        origin: None,
        whole_body_completion: None,
        response_evidence: None,
        request_started: true,
    });
    keeper.note_probe(&ProbeObservation {
        post,
        url: "still not a URL".to_owned(),
        outcome: Ok(ProbeResult {
            final_url: "still not a URL".to_owned(),
            observed: 1.into(),
            content_length: Some(1),
            accept_ranges: Some(true),
            content_type: Some("video/mp4".to_owned()),
            validator: None,
            ttfb: Duration::from_millis(1),
        }),
        concurrency: 1,
        network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
    });

    keeper.save_now().await;

    assert!(!path.exists());
    std::fs::remove_dir_all(root).expect("remove test directory");
}
