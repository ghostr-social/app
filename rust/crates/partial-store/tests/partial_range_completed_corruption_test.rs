mod store_fixture;

use std::io::{Seek, SeekFrom, Write};
use store_fixture::{discard, limits, reopened, spaced_store};

#[tokio::test]
async fn completed_object_corruption_is_never_served_after_restart() {
    let first = spaced_store("ghostr-completed-corruption", limits(64, 0), 64);
    first
        .store
        .write_range("clip", 0, b"abcdefgh")
        .await
        .unwrap();
    first.store.set_total_len("clip", 8).await.unwrap();
    first.store.finalize("clip", None).await.unwrap();

    corrupt_byte(&first.root.join("clip.video"));
    let second = reopened(&first);
    second.store.load_existing().await.expect("reload cache");
    assert_eq!(second.store.used_bytes().await, 8);

    assert_eq!(second.store.read_range("clip", 0..8).await.unwrap(), None);
    assert_eq!(second.store.used_bytes().await, 0);
    assert!(!second.root.join("clip.video").exists());
    discard(&second.root);
}

fn corrupt_byte(path: &std::path::Path) {
    let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.seek(SeekFrom::Start(3)).unwrap();
    file.write_all(b"X").unwrap();
    file.sync_all().unwrap();
}
