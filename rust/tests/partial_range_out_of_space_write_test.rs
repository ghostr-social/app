mod store_space;

use store_space::{discard, limits, spaced_store};

/// Running out of room mid-transfer must cost only the chunk that could
/// not land: the partial file, its manifest and its accounting stay
/// exactly as they were, and the video resumes once space returns.
#[tokio::test]
async fn partial_range_out_of_space_write_leaves_the_manifest_consistent() {
    let fixture = spaced_store("ghostr-out-of-space", limits(1_000_000, 1_000), 3_000);
    let store = &fixture.store;
    store.write_range("clip", 0, &[3; 400]).await.expect("head");
    store
        .set_total_len("clip", 4_000)
        .await
        .expect("total length");
    let manifest = fixture.root.join("clip.ranges.json");
    let before = std::fs::read_to_string(&manifest).expect("manifest");

    fixture.space.set(1_000);
    let refused = store
        .write_range("clip", 400, &[4; 400])
        .await
        .expect_err("no room for the next chunk");

    assert!(
        refused.to_string().contains("space"),
        "unhelpful: {refused}"
    );
    assert_eq!(
        std::fs::read_to_string(&manifest).expect("manifest"),
        before,
        "the manifest must not record bytes that never landed"
    );
    assert_eq!(
        store.present_ranges("clip").await.expect("ranges"),
        vec![0..400]
    );
    assert_eq!(store.total_len("clip").await.expect("total"), Some(4_000));
    assert!(!store.is_complete("clip").await.expect("completeness"));
    assert_eq!(*fixture.used_bytes.lock().await, 400);
    assert_eq!(
        store.read_range("clip", 0..400).await.expect("read head"),
        Some(vec![3; 400]),
        "the bytes already stored stay readable"
    );

    fixture.space.set(10_000);
    store
        .write_range("clip", 400, &[4; 400])
        .await
        .expect("resume once space returns");
    assert_eq!(
        store.present_ranges("clip").await.expect("ranges"),
        vec![0..800]
    );

    discard(&fixture.root);
}
