use super::whole_sink_fixture::{fixture, split, whole_spec, AuthorizedTraffic, FailingFinishSink};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::{download_chunk_observed, ChunkExecution};
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::host_stats::HostStats;

#[tokio::test]
async fn cap_stop_with_failed_rollback_is_a_local_store_failure() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nTransfer-Encoding: chunked\r\n\r\n9\r\nnew body!\r\n0\r\n\r\n";
    let origin = split(response, b"").await;
    origin.release.notify_one();
    let fixture = fixture("whole-rollback-failure", &origin.url, None).await;
    let (_handle, cancel) = cancel_pair();
    let mut stats = HostStats::new();
    let mut traffic = AuthorizedTraffic::new(
        std::sync::Arc::clone(&fixture.store),
        fixture.identity.clone(),
        fixture.action.clone(),
    );
    let spec = whole_spec(
        &fixture.client,
        &origin.url,
        WholeBodyContract::Capped { maximum_bytes: 8 },
    );

    let error = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &FailingFinishSink,
            stats: &mut stats,
            cancel: &cancel,
            network: &NetworkThrottle::new(),
            traffic: &mut traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    )
    .await
    .result
    .expect_err("scenario must fail");

    assert!(crate::chunk::sink::is_local_store_failure(&error));
    assert!(format!("{error:#}").contains("policy limit"));
    fixture.store.release_action(&fixture.action).await;
    std::fs::remove_dir_all(fixture.root).expect("valid test fixture");
}
