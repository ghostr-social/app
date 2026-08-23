use super::whole_sink_fixture::{fixture, split, whole_spec, AuthorizedTraffic};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::{download_chunk_captured, ChunkExecution};
use crate::chunk::sink::TransferChunkSink;
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::origin_model::OriginOutcome;
use std::time::Duration;

#[tokio::test]
async fn oversized_content_length_is_learned_without_reading_or_blame() {
    let headers = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 9\r\n\r\n";
    let origin = split(headers, b"new body!").await;
    let fixture = fixture("whole-content-length-bound", &origin.url, Some(b"old!")).await;
    let sink = TransferChunkSink::new(
        &fixture.store,
        fixture.identity.clone(),
        fixture.action.clone(),
    );
    let (_handle, cancel) = cancel_pair();
    let mut stats = HostStats::new();
    let mut traffic = AuthorizedTraffic::new(
        fixture.store.clone(),
        fixture.identity.clone(),
        fixture.action.clone(),
    );
    let spec = whole_spec(
        &fixture.client,
        &origin.url,
        WholeBodyContract::Capped { maximum_bytes: 8 },
    );

    let observed = tokio::time::timeout(
        Duration::from_secs(1),
        download_chunk_captured(
            &spec,
            ChunkExecution {
                sink: &sink,
                stats: &mut stats,
                cancel: &cancel,
                network: &NetworkThrottle::new(),
                traffic: &mut traffic,
                network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
            },
        ),
    )
    .await
    .expect("headers alone resolve the bounded probe");

    let error = observed.result.unwrap_err();
    let bound = crate::chunk::whole_body_bound::from_error(&error).unwrap();
    assert_eq!((bound.maximum_bytes(), bound.total_bytes()), (8, 9));
    assert_eq!(observed.received_bytes, 0);
    assert!(matches!(observed.origin.unwrap().outcome, OriginOutcome::Success));
    assert_eq!(fixture.store.read_range("post", 0..4).await.unwrap(), Some(b"old!".to_vec()));
    origin.release.notify_one();
    fixture.store.release_action(&fixture.action).await;
    std::fs::remove_dir_all(fixture.root).unwrap();
}
