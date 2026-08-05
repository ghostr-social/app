use super::support::temp_directory;
use crate::engine::PostId;
use crate::video::delivery_stats::StatsKeeper;
use crate::video::delivery_transfers::ProbeDone;
use anyhow::anyhow;
use std::time::Duration;

#[tokio::test]
async fn failed_host_stats_save_stays_dirty_for_the_next_attempt() {
    let root = temp_directory("ghostr-stats-save-retry");
    let parent = root.join("not-created");
    let path = parent.join("host_stats.json");
    let mut keeper = StatsKeeper::load(path.clone(), Duration::ZERO).await;
    keeper.note_probe(&ProbeDone {
        post: PostId::new("clip"),
        url: "https://media.example/clip.mp4".to_owned(),
        outcome: Err(anyhow!("origin failed")),
    });

    keeper.save_now().await;
    assert!(!path.exists());

    std::fs::create_dir(&parent).expect("create stats directory");
    keeper.save_now().await;

    assert!(path.exists(), "dirty snapshot should retry");
    std::fs::remove_dir_all(root).expect("remove test directory");
}
