mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;

#[tokio::test]
async fn additive_bytes_keep_the_same_progressive_asset() {
    let harness = progressive_harness("ghostr-progressive-asset-growth");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(8))
        .await;
    harness
        .store
        .set_total_len("clip", 8)
        .await
        .expect("valid test fixture");
    harness
        .store
        .write_range("clip", 0, b"aaaa")
        .await
        .expect("valid test fixture");
    let first = harness.issue_video_asset("clip").await;

    harness
        .store
        .write_range("clip", 4, b"bbbb")
        .await
        .expect("valid test fixture");
    let second = harness.issue_video_asset("clip").await;

    assert_eq!(first, second);
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
