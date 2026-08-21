mod range_fixture;
mod raw_http;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use std::time::Duration;

#[tokio::test]
async fn cancellation_while_waiting_for_headers_ends_the_request_promptly() {
    let stalled = raw_http::spawn_stalled_headers().await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-header-cancel");
    let store = range_fixture::store(root.clone());
    let (handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &stalled.url,
        request: range_fixture::range_request(ByteRange::new(0, 8)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts {
            admission: Duration::from_secs(30),
            headers: Duration::from_secs(30),
            idle: Duration::from_secs(30),
        },
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let network = range_fixture::network();
    let cancel = async {
        stalled.request_started.await.unwrap();
        handle.cancel();
    };

    let (result, ()) = tokio::time::timeout(Duration::from_secs(1), async {
        tokio::join!(
            range_fixture::download_chunk_throttled(
                &spec,
                &sink,
                range_fixture::context(&mut stats, &token, &network)
            ),
            cancel,
        )
    })
    .await
    .expect("cancellation deadline");

    assert!(result.unwrap().cancelled);
    stalled.requests.await.unwrap();
    std::fs::remove_dir_all(root).ok();
}
