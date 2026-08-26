use std::os::unix::fs::PermissionsExt as _;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn an_incomplete_store_scan_fails_closed_before_admission() {
    let root = crate::tests::store_fixture::temp_root("reload-enumeration-failure");
    tokio::fs::create_dir_all(&root)
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("post.part"), b"unaccounted")
        .await
        .expect("valid test fixture");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o0))
        .expect("valid test fixture");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));

    let result = store.load_existing().await;

    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
        .expect("valid test fixture");
    assert!(result.is_err());
    crate::tests::store_fixture::discard(&root);
}
