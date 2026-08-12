use super::support::temp_directory;
use crate::manager::FocusedStoreLease;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

struct RoomyDisk;

impl FreeSpace for RoomyDisk {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        Some(10_000)
    }
}

#[tokio::test]
async fn focused_video_survives_capacity_eviction_between_player_requests() {
    let root = temp_directory("ghostr-focused-store-lease");
    let capacity = StoreCapacity::new(
        Limits {
            budget: 800,
            reserve: 0,
        },
        Arc::new(RoomyDisk),
        Duration::ZERO,
    );
    let store = PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity);
    store
        .write_range("focused", 0, &[1; 400])
        .await
        .expect("focused");
    store.write_range("old", 0, &[2; 400]).await.expect("old");
    let mut lease = FocusedStoreLease::default();
    lease.pin(&store, Some(&PostId::new("focused")));

    store.set_storage_budget(400).await.expect("shrink budget");

    assert_eq!(store.present_ranges("focused").await.unwrap(), vec![0..400]);
    assert!(store.present_ranges("old").await.unwrap().is_empty());
    drop(lease);
    std::fs::remove_dir_all(root).expect("remove test directory");
}
