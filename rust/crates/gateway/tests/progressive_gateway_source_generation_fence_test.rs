mod gateway_fixture;

use core::time::Duration;
use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use tokio_stream::StreamExt as _;
use tower::ServiceExt as _;

#[tokio::test]
async fn open_gateway_response_never_splices_two_source_generations() {
    let harness = progressive_harness("ghostr-source-generation-fence");
    harness.posts.insert("clip");
    let total = PLAYBACK_SLICE_BYTES * 2;
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta(total));
    let transfer = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    harness
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    harness
        .store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    harness
        .store
        .accept_generation(&transfer, generation(total))
        .await
        .expect("valid test fixture");
    harness
        .store
        .set_total_len("clip", total)
        .await
        .expect("valid test fixture");
    harness
        .store
        .write_range("clip", 0, &vec![b'a'; PLAYBACK_SLICE_BYTES as usize])
        .await
        .expect("valid test fixture");
    let end = total - 1;
    let request = harness
        .video_request("clip", Some(&format!("bytes=0-{end}")))
        .await;
    let response = harness
        .router
        .oneshot(request)
        .await
        .expect("valid test fixture");
    let mut body = response.into_body().into_data_stream();
    let first = body
        .next()
        .await
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert!(first.iter().all(|byte| *byte == b'a'));

    harness
        .store
        .begin_single_response(
            &transfer,
            7,
            WholeBodyContract::Exact {
                expected_bytes: total,
            },
        )
        .await
        .expect("valid test fixture");
    harness
        .store
        .write_single_response_if_current(&transfer, 7, 0, &vec![b'b'; total as usize])
        .await
        .expect("valid test fixture");
    assert!(harness
        .store
        .finish_single_response(&transfer, 7, Some(total), true)
        .await
        .expect("valid test fixture"));

    let stopped = tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("generation switch terminates the old stream")
        .expect("body termination");
    assert!(stopped.is_err());
    std::fs::remove_dir_all(harness.root).expect("remove store");
}

fn generation(total: u64) -> SourceGeneration {
    SourceGeneration::try_new("https://cdn.example/video", "\"old\"", total)
        .expect("valid test fixture")
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
