use sha2::{Digest as _, Sha256};

const MAX_WARP_CANCELLATION_BLOCK: u64 = 512 * 1024;
const TOO_LARGE: u64 = MAX_WARP_CANCELLATION_BLOCK + 1;

#[tokio::test]
async fn the_largest_warp_checksum_interval_remains_readable() {
    let fixture = fixture("manifest-interval-maximum", MAX_WARP_CANCELLATION_BLOCK);
    let bytes = vec![0; MAX_WARP_CANCELLATION_BLOCK as usize];
    seed(&fixture.root, &bytes, &digest(&bytes));

    fixture
        .store
        .load_existing()
        .await
        .expect("valid test fixture");
    assert_eq!(
        fixture.store.used_bytes().await,
        MAX_WARP_CANCELLATION_BLOCK
    );
    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..MAX_WARP_CANCELLATION_BLOCK)
            .await
            .expect("valid test fixture"),
        Some(bytes)
    );
    crate::tests::store_fixture::discard(&fixture.root);
}

#[tokio::test]
async fn an_oversized_checksum_interval_is_never_loaded_or_allocated() {
    let fixture = fixture("manifest-interval-limit", TOO_LARGE);
    seed(&fixture.root, &vec![0; TOO_LARGE as usize], &"0".repeat(64));

    fixture
        .store
        .load_existing()
        .await
        .expect("valid test fixture");
    assert!(fixture
        .store
        .present_ranges("clip")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert_eq!(fixture.store.used_bytes().await, 0);
    assert!(!fixture.root.join("clip.part").exists());
    crate::tests::store_fixture::discard(&fixture.root);
}

fn fixture(prefix: &str, len: u64) -> crate::tests::store_fixture::SpacedStore {
    crate::tests::store_fixture::spaced_store(
        prefix,
        crate::tests::store_fixture::limits(len * 2, 0),
        len * 2,
    )
}

fn seed(root: &std::path::Path, bytes: &[u8], sha256: &str) {
    std::fs::create_dir_all(root).expect("valid test fixture");
    std::fs::write(root.join("clip.part"), bytes).expect("valid test fixture");
    let len = bytes.len();
    let manifest = format!(
        r#"{{"version":2,"total_len":{len},"intervals":[{{"start":0,"end":{len},"sha256":"{sha256}"}}]}}"#
    );
    std::fs::write(root.join("clip.ranges.json"), manifest).expect("valid test fixture");
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
