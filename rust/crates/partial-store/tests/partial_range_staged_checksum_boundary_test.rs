use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::io::{Seek as _, SeekFrom, Write as _};
use std::sync::Arc;
use tokio::sync::Mutex;

const TOTAL: usize = 640 * 1024;
const CORRUPT_AT: u64 = 576 * 1024;

#[tokio::test]
async fn staged_whole_corruption_does_not_poison_an_unrelated_action_interval() {
    let root = crate::tests::store_fixture::temp_root("staged-checksum-boundary");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"old!")
        .await
        .expect("valid test fixture");
    store
        .begin_single_response(
            &transfer,
            1,
            crate::tests::store_fixture::exact_response(TOTAL as u64),
        )
        .await
        .expect("valid test fixture");
    store
        .write_single_response_if_current(&transfer, 1, 0, &vec![b'v'; TOTAL])
        .await
        .expect("valid test fixture");
    store
        .finish_single_response(&transfer, 1, Some(TOTAL as u64), true)
        .await
        .expect("valid test fixture");

    corrupt(&root.join("post.part"));
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"vvvv".to_vec())
    );
    assert_eq!(
        store
            .read_range("post", CORRUPT_AT..CORRUPT_AT + 4)
            .await
            .expect("valid test fixture"),
        None
    );
    crate::tests::store_fixture::discard(&root);
}

fn corrupt(path: &std::path::Path) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("valid test fixture");
    file.seek(SeekFrom::Start(CORRUPT_AT))
        .expect("valid test fixture");
    file.write_all(b"X").expect("valid test fixture");
    file.sync_all().expect("valid test fixture");
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(TOTAL as u64),
        duration_ms: Some(1_000),
    }
}
