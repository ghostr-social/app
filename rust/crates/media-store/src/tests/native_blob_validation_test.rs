use super::support::{cached, temp_directory};
use crate::native_blob_integrity::{validate_blob, NativeBlobSnapshot};
use ghostr_media_model::native_models::NativeVideoCacheKey;

const VIDEO_DIGEST: &str = "0cab1c9617404faf2b24e221e189ca5945813e14d3f766345b09ca13bbe28ffc";

#[tokio::test]
async fn native_blob_validation_checks_shape_length_and_identity() {
    let root = temp_directory("ghostr-blob-validation");
    let path = root.join("clip.mp4");
    tokio::fs::write(&path, b"video").await.expect("write blob");
    let modified = tokio::fs::metadata(&path)
        .await
        .expect("metadata")
        .modified()
        .ok();
    let url_snapshot = NativeBlobSnapshot {
        key: NativeVideoCacheKey::UrlDerived("a".repeat(64)),
        modified: None,
        video: cached(&path, 5),
    };
    assert!(
        validate_blob(&url_snapshot)
            .await
            .expect("URL validation")
            .valid
    );

    let digest_snapshot = NativeBlobSnapshot {
        key: NativeVideoCacheKey::AdvertisedDigest(VIDEO_DIGEST.to_owned()),
        modified: None,
        video: cached(&path, 5),
    };
    assert!(validate_blob(&digest_snapshot).await.expect("digest").valid);
    let unchanged_snapshot = NativeBlobSnapshot {
        key: NativeVideoCacheKey::AdvertisedDigest("0".repeat(64)),
        modified,
        video: cached(&path, 5),
    };
    assert!(
        validate_blob(&unchanged_snapshot)
            .await
            .expect("unchanged")
            .valid
    );
    let bad_digest = NativeBlobSnapshot {
        key: NativeVideoCacheKey::AdvertisedDigest("0".repeat(64)),
        modified: None,
        video: cached(&path, 5),
    };
    assert!(!validate_blob(&bad_digest).await.expect("bad digest").valid);

    let wrong_length = NativeBlobSnapshot {
        key: NativeVideoCacheKey::UrlDerived("b".repeat(64)),
        modified: None,
        video: cached(&path, 6),
    };
    assert!(!validate_blob(&wrong_length).await.expect("length").valid);
    let directory = root.join("directory");
    tokio::fs::create_dir(&directory).await.expect("directory");
    let wrong_kind = NativeBlobSnapshot {
        key: NativeVideoCacheKey::UrlDerived("c".repeat(64)),
        modified: None,
        video: cached(&directory, 0),
    };
    assert!(!validate_blob(&wrong_kind).await.expect("kind").valid);
    let missing = NativeBlobSnapshot {
        key: NativeVideoCacheKey::UrlDerived("d".repeat(64)),
        modified: None,
        video: cached(&root.join("missing.mp4"), 0),
    };
    assert!(!validate_blob(&missing).await.expect("missing").valid);

    let plain = root.join("plain");
    tokio::fs::write(&plain, []).await.expect("plain file");
    let inaccessible = NativeBlobSnapshot {
        key: NativeVideoCacheKey::UrlDerived("e".repeat(64)),
        modified: None,
        video: cached(&plain.join("child"), 0),
    };
    assert!(validate_blob(&inaccessible).await.is_err());
    std::fs::remove_dir_all(root).expect("remove test directory");
}
