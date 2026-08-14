use ghostr_partial_store::partial_range_store::{OutOfSpace, PartialRangeStore};

pub(crate) async fn fill_and_refuse(store: &PartialRangeStore) -> OutOfSpace {
    store.write_range("aa11", 0, &[1; 8]).await.unwrap();
    store
        .write_range("aa11", 8, &[2; 8])
        .await
        .unwrap_err()
        .downcast::<OutOfSpace>()
        .unwrap()
}
