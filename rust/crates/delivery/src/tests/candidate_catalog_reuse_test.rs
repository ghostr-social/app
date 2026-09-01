use super::candidate_catalog_fixture::{binding, candidate, url};
use super::support::temp_directory;
use crate::manager::state::DeliveryState;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DataUsageLevel, EngineParams, PostId};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn readmitted_candidate_reuses_sparse_bytes_after_catalog_eviction() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let (store, root) = store();
    state.apply_candidate(candidate("reused", 0));
    let first = binding(&mut state, "reused");
    store
        .bind_representation(first.clone())
        .await
        .expect("valid test fixture");
    let identity = first.transfer(&url("reused")).expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    let generation =
        SourceGeneration::try_new(url("reused"), "\"stable\"", 8).expect("valid test fixture");
    store
        .accept_generation(&identity, generation.clone())
        .await
        .expect("valid test fixture");
    assert!(store
        .write_range_for_generation_if_current(&identity, &generation, 0, &[1; 4])
        .await
        .expect("valid test fixture"));

    for index in 1..=64 {
        state.apply_candidate(candidate(&format!("new-{index}"), index));
        state.take_representation_bindings();
    }
    assert!(state.catalog().lookup(&PostId::new("reused")).is_none());
    assert_eq!(
        store
            .present_ranges("reused")
            .await
            .expect("valid test fixture"),
        vec![0..4]
    );

    state.apply_candidate(candidate("reused", 100));
    let readmitted = binding(&mut state, "reused");
    assert_ne!(readmitted, first);
    store
        .bind_representation(readmitted)
        .await
        .expect("valid test fixture");
    assert!(!store
        .write_range_for_transfer_if_current(&identity, 4, &[9; 4])
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .present_ranges("reused")
            .await
            .expect("valid test fixture"),
        vec![0..4]
    );
    std::fs::remove_dir_all(root).expect("valid test fixture");
}

fn store() -> (PartialRangeStore, std::path::PathBuf) {
    let root = temp_directory("ghostr-candidate-reuse");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    (store, root)
}
