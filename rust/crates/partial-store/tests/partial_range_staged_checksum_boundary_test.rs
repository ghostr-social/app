mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::io::{Seek, SeekFrom, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

const TOTAL: usize = 640 * 1024;
const CORRUPT_AT: u64 = 576 * 1024;

#[tokio::test]
async fn staged_whole_corruption_does_not_poison_an_unrelated_action_interval() {
    let root = store_fixture::temp_root("staged-checksum-boundary");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    store.write_range("post", 0, b"old!").await.unwrap();
    store
        .begin_single_response(&transfer, 1, store_fixture::exact_response(TOTAL as u64))
        .await
        .unwrap();
    store
        .write_single_response_if_current(&transfer, 1, 0, &vec![b'v'; TOTAL])
        .await
        .unwrap();
    store
        .finish_single_response(&transfer, 1, Some(TOTAL as u64), true)
        .await
        .unwrap();

    corrupt(&root.join("post.part"));
    assert_eq!(
        store.read_range("post", 0..4).await.unwrap(),
        Some(b"vvvv".to_vec())
    );
    assert_eq!(
        store
            .read_range("post", CORRUPT_AT..CORRUPT_AT + 4)
            .await
            .unwrap(),
        None
    );
    store_fixture::discard(&root);
}

fn corrupt(path: &std::path::Path) {
    let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.seek(SeekFrom::Start(CORRUPT_AT)).unwrap();
    file.write_all(b"X").unwrap();
    file.sync_all().unwrap();
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
