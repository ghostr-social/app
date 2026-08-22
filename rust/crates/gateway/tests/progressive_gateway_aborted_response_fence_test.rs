mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::time::Duration;
use tokio_stream::StreamExt;
use tower::ServiceExt;

#[tokio::test]
async fn open_gateway_response_never_splices_an_aborted_response_with_its_retry() {
    let harness = progressive_harness("ghostr-aborted-response-fence");
    harness.posts.insert("clip");
    let total = PLAYBACK_SLICE_BYTES * 2;
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta(total));
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    harness.store.bind_representation(binding).await.unwrap();
    harness
        .store
        .select_transfer(transfer.clone())
        .await
        .unwrap();
    harness
        .store
        .begin_single_response(&transfer, 1, exact(total))
        .await
        .unwrap();
    harness
        .store
        .write_single_response_if_current(
            &transfer,
            1,
            0,
            &vec![b'a'; PLAYBACK_SLICE_BYTES as usize],
        )
        .await
        .unwrap();

    let request = harness
        .video_request("clip", Some(&format!("bytes=0-{}", total - 1)))
        .await;
    let response = harness.router.oneshot(request).await.unwrap();
    let mut body = response.into_body().into_data_stream();
    let first = body.next().await.unwrap().unwrap();
    assert!(first.iter().all(|byte| *byte == b'a'));

    assert!(!harness
        .store
        .finish_single_response(&transfer, 1, None, false)
        .await
        .unwrap());
    assert!(!harness
        .store
        .begin_single_response(&transfer, 2, exact(total))
        .await
        .unwrap());
    harness
        .store
        .select_transfer(transfer.clone())
        .await
        .unwrap();
    assert!(harness
        .store
        .begin_single_response(&transfer, 2, exact(total))
        .await
        .unwrap());
    harness
        .store
        .write_single_response_if_current(&transfer, 2, 0, &vec![b'b'; total as usize])
        .await
        .unwrap();

    let stopped = tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("aborting the response terminates the old stream")
        .expect("body termination");
    assert!(stopped.is_err());
    std::fs::remove_dir_all(harness.root).expect("remove store");
}

fn meta(total: u64) -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(total),
        duration_ms: Some(1_000),
    }
}

fn exact(expected_bytes: u64) -> WholeBodyContract {
    WholeBodyContract::Exact { expected_bytes }
}
