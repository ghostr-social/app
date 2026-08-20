mod store_fixture;

use std::sync::Arc;
use store_fixture::{discard, plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn integrity_quarantine_immediately_removes_every_cached_byte_for_a_key() {
    let root = temp_root("integrity-quarantine");
    let used = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), used.clone());
    store.write_range("mirror", 0, b"untrusted").await.unwrap();
    let before = store.media_snapshot("mirror").await.unwrap().revision();

    store.quarantine("mirror").await.unwrap();

    assert_eq!(store.read_range("mirror", 0..9).await.unwrap(), None);
    assert!(!root.join("mirror.part").exists());
    assert_eq!(*used.lock().await, 0);
    assert_ne!(
        store.media_snapshot("mirror").await.unwrap().revision(),
        before
    );
    discard(&root);
}
