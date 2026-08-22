mod store_fixture;

use std::io::{Seek, SeekFrom, Write};

const BLOCK: usize = ghostr_engine::adaptive::REQUEST_SLICE_BYTES as usize;
const OVERLAP: usize = BLOCK / 2;
const CORRUPT_AT: u64 = (BLOCK + OVERLAP / 2) as u64;

#[tokio::test]
async fn overlapping_sparse_writes_keep_corruption_inside_one_action_block() {
    let fixture = store_fixture::spaced_store(
        "sparse-checksum-boundary",
        store_fixture::limits((BLOCK * 2) as u64, 0),
        (BLOCK * 2) as u64,
    );
    fixture
        .store
        .write_range("clip", 0, &vec![b'a'; BLOCK])
        .await
        .unwrap();
    fixture
        .store
        .write_range("clip", OVERLAP as u64, &vec![b'b'; BLOCK])
        .await
        .unwrap();

    corrupt(&fixture.root.join("clip.part"));
    assert_eq!(
        fixture.store.read_range("clip", 0..4).await.unwrap(),
        Some(b"aaaa".to_vec())
    );
    assert_eq!(
        fixture
            .store
            .read_range("clip", CORRUPT_AT..CORRUPT_AT + 4)
            .await
            .unwrap(),
        None
    );
    store_fixture::discard(&fixture.root);
}

fn corrupt(path: &std::path::Path) {
    let mut file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.seek(SeekFrom::Start(CORRUPT_AT)).unwrap();
    file.write_all(b"X").unwrap();
    file.sync_all().unwrap();
}
