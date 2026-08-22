#[path = "store_fixture/paused.rs"]
mod paused_fixture;
mod store_fixture;

use sha2::{Digest, Sha256};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn policy_copy_never_blesses_bytes_changed_after_verification() {
    let mut fixture = paused_fixture::paused_store("policy-verified-copy");
    seed_video(&fixture.root).await;
    fixture.store.load_existing().await.unwrap();
    let store = fixture.store.clone();
    let eviction = tokio::spawn(async move {
        store
            .evict_ranges("clip", std::slice::from_ref(&(4..8)))
            .await
    });
    fixture.wait_until_admission().await;
    corrupt_first_byte(&fixture.root.join("clip.part")).await;
    fixture.resume();

    assert!(eviction.await.unwrap().is_err());
    assert_eq!(fixture.store.read_range("clip", 0..4).await.unwrap(), None);
    assert_eq!(fixture.store.used_bytes().await, 0);
    store_fixture::discard(&fixture.root);
}

async fn seed_video(root: &std::path::Path) {
    let bytes = b"abcdefghijkl";
    tokio::fs::create_dir_all(root).await.unwrap();
    tokio::fs::write(root.join("clip.part"), bytes)
        .await
        .unwrap();
    let digest = format!("{:x}", Sha256::digest(bytes));
    let manifest = format!(
        r#"{{"version":2,"total_len":12,"intervals":[{{"start":0,"end":12,"sha256":"{digest}"}}]}}"#
    );
    tokio::fs::write(root.join("clip.ranges.json"), manifest)
        .await
        .unwrap();
}

async fn corrupt_first_byte(path: &std::path::Path) {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .await
        .unwrap();
    file.seek(std::io::SeekFrom::Start(0)).await.unwrap();
    file.write_all(b"X").await.unwrap();
    file.sync_all().await.unwrap();
}
