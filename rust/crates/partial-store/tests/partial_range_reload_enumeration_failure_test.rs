mod store_fixture;

use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn an_incomplete_store_scan_fails_closed_before_admission() {
    let root = store_fixture::temp_root("reload-enumeration-failure");
    tokio::fs::create_dir_all(&root).await.unwrap();
    tokio::fs::write(root.join("post.part"), b"unaccounted")
        .await
        .unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o0)).unwrap();
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));

    let result = store.load_existing().await;

    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err());
    store_fixture::discard(&root);
}
