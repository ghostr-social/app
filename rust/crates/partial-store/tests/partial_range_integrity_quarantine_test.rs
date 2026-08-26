use crate::tests::store_fixture::{discard, plain_store, temp_root};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn integrity_quarantine_immediately_removes_every_cached_byte_for_a_key() {
    let root = temp_root("integrity-quarantine");
    let used = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), std::sync::Arc::clone(&used));
    store
        .write_range("mirror", 0, b"untrusted")
        .await
        .expect("valid test fixture");
    let before = store
        .media_snapshot("mirror")
        .await
        .expect("valid test fixture")
        .revision();

    store
        .quarantine("mirror")
        .await
        .expect("valid test fixture");

    assert_eq!(
        store
            .read_range("mirror", 0..9)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(!root.join("mirror.part").exists());
    assert_eq!(*used.lock().await, 0);
    assert_ne!(
        store
            .media_snapshot("mirror")
            .await
            .expect("valid test fixture")
            .revision(),
        before
    );
    discard(&root);
}
