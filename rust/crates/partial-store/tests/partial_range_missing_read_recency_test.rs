use crate::tests::store_fixture::{discard, limits, spaced_store};

#[tokio::test]
async fn a_missing_read_does_not_protect_cold_bytes_from_eviction() {
    let fixture = spaced_store("missing-read-recency", limits(800, 0), 8_000);
    let store = &fixture.store;
    store
        .write_range("older", 0, &[1; 400])
        .await
        .expect("valid test fixture");
    store
        .write_range("newer", 0, &[2; 400])
        .await
        .expect("valid test fixture");

    assert_eq!(
        store
            .read_range("older", 400..401)
            .await
            .expect("valid test fixture"),
        None
    );
    store
        .write_range("incoming", 0, &[3; 400])
        .await
        .expect("valid test fixture");

    assert!(store
        .present_ranges("older")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert_eq!(
        store
            .present_ranges("newer")
            .await
            .expect("valid test fixture"),
        vec![0..400]
    );
    assert_eq!(
        store
            .present_ranges("incoming")
            .await
            .expect("valid test fixture"),
        vec![0..400]
    );
    discard(&fixture.root);
}
