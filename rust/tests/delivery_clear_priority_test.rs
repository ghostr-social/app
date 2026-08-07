mod support;

use rust_lib_ghostr::engine::{DeliveryKind, PostId, VideoMeta};
use rust_lib_ghostr::video::delivery_events::DeliveryCandidate;
use std::time::Duration;
use support::delivery::start_harness;
use support::delivery_options::DeliveryOptions;

#[tokio::test]
async fn clear_jumps_a_large_candidate_backlog() {
    let harness = start_harness("ghostr-clear-priority", DeliveryOptions::default());
    for sequence in 0..1_000 {
        harness.handle.admit_candidate(candidate(sequence));
    }

    tokio::time::timeout(Duration::from_secs(1), harness.handle.clear())
        .await
        .expect("clear must not wait behind candidate ingestion")
        .expect("clear delivery");

    assert!(harness.cache.videos().is_empty());
    assert_eq!(harness.store.used_bytes().await, 0);
}

fn candidate(sequence: u64) -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new(format!("candidate-{sequence}")),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{sequence}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
        discovered_at: sequence,
    }
}
