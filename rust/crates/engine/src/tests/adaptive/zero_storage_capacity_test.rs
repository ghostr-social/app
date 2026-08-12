use crate::adaptive::StorageSnapshot;

#[test]
fn zero_effective_capacity_has_no_available_bytes() {
    let storage = StorageSnapshot::new(0, 0);

    assert_eq!(storage.available_bytes(), 0);
}
