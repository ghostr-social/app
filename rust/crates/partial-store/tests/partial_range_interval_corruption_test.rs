mod store_fixture;

use std::io::{Seek, SeekFrom, Write};
use store_fixture::{limits, reopened, spaced_store};

#[tokio::test]
async fn committed_interval_corruption_is_never_served_after_restart() {
    let first = spaced_store("ghostr-interval-corruption", limits(64, 0), 64);
    first
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .expect("commit interval");
    first.store.set_total_len("clip", 8).await.expect("total");

    let second = reopened(&first);
    second.store.load_existing().await.expect("reload cache");
    assert_eq!(
        second.store.present_ranges("clip").await.unwrap(),
        vec![0..8]
    );
    assert_eq!(second.store.used_bytes().await, 8);
    assert_eq!(
        second.store.read_range("clip", 0..8).await.unwrap(),
        Some(b"abcdefgh".to_vec())
    );
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(second.root.join("clip.part"))
        .expect("open stored interval");
    file.seek(SeekFrom::Start(3)).expect("seek corruption");
    file.write_all(b"X").expect("corrupt one byte");
    file.sync_all().expect("persist corruption");

    assert_eq!(
        second.store.read_range("clip", 0..8).await.expect("read"),
        None,
        "a checksum mismatch must never reach the gateway"
    );
    assert_eq!(second.store.used_bytes().await, 0);
    assert!(!second.root.join("clip.part").exists());
    store_fixture::discard(&second.root);
}
