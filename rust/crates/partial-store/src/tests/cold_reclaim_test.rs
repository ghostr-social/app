use crate::tests::store_fixture::{discard, limits, spaced_store};
use std::collections::HashSet;

#[tokio::test]
async fn cold_reclaim_preserves_the_working_set_and_read_leases() {
    let fixture = spaced_store("ghostr-cold-reclaim-protection", limits(1_200, 0), 10_000);
    let store = &fixture.store;
    store
        .write_range("current", 0, &[1; 400])
        .await
        .expect("valid test fixture");
    store
        .write_range("leased", 0, &[2; 400])
        .await
        .expect("valid test fixture");
    store
        .write_range("cold", 0, &[3; 400])
        .await
        .expect("valid test fixture");
    let lease = store.lease("leased");
    let working_set = HashSet::from(["current".to_owned()]);

    assert_eq!(store.reclaim_outside(&working_set, 800).await, 400);
    assert_eq!(
        store
            .read_range("current", 0..400)
            .await
            .expect("valid test fixture"),
        Some(vec![1; 400])
    );
    assert_eq!(
        store
            .read_range("leased", 0..400)
            .await
            .expect("valid test fixture"),
        Some(vec![2; 400])
    );
    assert!(store
        .present_ranges("cold")
        .await
        .expect("valid test fixture")
        .is_empty());
    drop(lease);
    assert_eq!(store.reclaim_outside(&working_set, 800).await, 400);
    assert!(store
        .present_ranges("leased")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert_eq!(store.reclaim_outside(&working_set, 800).await, 0);
    discard(&fixture.root);
}
