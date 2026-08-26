use std::sync::Arc;
use tokio::io::{AsyncSeekExt as _, AsyncWriteExt as _};
use tokio::sync::Mutex;

#[tokio::test]
async fn policy_eviction_never_blesses_corrupt_retained_bytes() {
    let root = crate::tests::store_fixture::temp_root("policy-integrity");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("clip", 12)
        .await
        .expect("valid test fixture");
    corrupt_first_byte(&root.join("clip.part")).await;

    let result = store
        .evict_ranges("clip", core::slice::from_ref(&(8..12)))
        .await;

    assert!(result.is_err());
    assert_eq!(
        store
            .read_range("clip", 0..4)
            .await
            .expect("valid test fixture"),
        None
    );
    assert_eq!(store.used_bytes().await, 0);
    crate::tests::store_fixture::discard(&root);
}

async fn corrupt_first_byte(path: &std::path::Path) {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .await
        .expect("valid test fixture");
    file.seek(std::io::SeekFrom::Start(0))
        .await
        .expect("valid test fixture");
    file.write_all(b"X").await.expect("valid test fixture");
    file.sync_all().await.expect("valid test fixture");
}
