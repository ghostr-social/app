use std::io::{Seek as _, SeekFrom, Write as _};

const BLOCK: usize = ghostr_engine::adaptive::REQUEST_SLICE_BYTES as usize;
const OVERLAP: usize = BLOCK / 2;
const CORRUPT_AT: u64 = (BLOCK + OVERLAP / 2) as u64;

#[tokio::test]
async fn overlapping_sparse_writes_keep_corruption_inside_one_action_block() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "sparse-checksum-boundary",
        crate::tests::store_fixture::limits((BLOCK * 2) as u64, 0),
        (BLOCK * 2) as u64,
    );
    fixture
        .store
        .write_range("clip", 0, &vec![b'a'; BLOCK])
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range("clip", OVERLAP as u64, &vec![b'b'; BLOCK])
        .await
        .expect("valid test fixture");

    corrupt(&fixture.root.join("clip.part"));
    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"aaaa".to_vec())
    );
    assert_eq!(
        fixture
            .store
            .read_range("clip", CORRUPT_AT..CORRUPT_AT + 4)
            .await
            .expect("valid test fixture"),
        None
    );
    crate::tests::store_fixture::discard(&fixture.root);
}

fn corrupt(path: &std::path::Path) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("valid test fixture");
    file.seek(SeekFrom::Start(CORRUPT_AT))
        .expect("valid test fixture");
    file.write_all(b"X").expect("valid test fixture");
    file.sync_all().expect("valid test fixture");
}
