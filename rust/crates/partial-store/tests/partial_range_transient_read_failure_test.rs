#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;

#[tokio::test]
async fn a_transient_read_error_does_not_delete_valid_cached_bytes() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "transient-read-failure",
        crate::tests::store_fixture::limits(16, 0),
        16,
    );
    fixture
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .expect("valid test fixture");
    let path = fixture.root.join("clip.part");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o0))
        .expect("valid test fixture");

    let error = fixture
        .store
        .read_range("clip", 0..8)
        .await
        .expect_err("scenario must fail");
    assert!(is_permission_denied(&error));
    assert_eq!(
        fixture
            .store
            .present_ranges("clip")
            .await
            .expect("valid test fixture"),
        vec![0..8]
    );
    assert_eq!(fixture.store.used_bytes().await, 8);
    assert!(path.exists());

    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
        .expect("valid test fixture");
    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefgh".to_vec())
    );
    crate::tests::store_fixture::discard(&fixture.root);
}

fn is_permission_denied(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io| io.kind() == std::io::ErrorKind::PermissionDenied)
    })
}
