use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use std::path::Path;
use std::sync::Arc;

struct UnknownSpace;

impl FreeSpace for UnknownSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        None
    }
}

#[tokio::test]
async fn unknown_free_space_falls_back_to_budget_without_a_standing_sample() {
    let capacity = StoreCapacity::new(
        Limits {
            budget: 800,
            reserve: 400,
        },
        Arc::new(UnknownSpace),
        std::time::Duration::ZERO,
    );
    capacity.gave_back(100).await;
    capacity.spent(100).await;

    assert_eq!(capacity.cap(Path::new("/not/consulted"), 200).await, 800);
}
