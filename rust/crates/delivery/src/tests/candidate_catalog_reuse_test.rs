use super::support::temp_directory;
use crate::delivery_events::DeliveryCandidate;
use crate::manager::state::DeliveryState;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
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
    store.bind_representation(first.clone()).await.unwrap();
    let identity = first.transfer(&url("reused")).unwrap();
    store.select_transfer(identity.clone());
    assert!(store
        .write_range_for_transfer_if_current(&identity, 0, &[1; 4])
        .await
        .unwrap());

    for index in 1..=64 {
        state.apply_candidate(candidate(&format!("new-{index}"), index));
        state.take_representation_bindings();
    }
    assert!(state.catalog().lookup(&PostId::new("reused")).is_none());
    assert_eq!(store.present_ranges("reused").await.unwrap(), vec![0..4]);

    state.apply_candidate(candidate("reused", 100));
    let readmitted = binding(&mut state, "reused");
    assert_ne!(readmitted.generation(), first.generation());
    store.bind_representation(readmitted).await.unwrap();
    assert!(!store
        .write_range_for_transfer_if_current(&identity, 4, &[9; 4])
        .await
        .unwrap());
    assert_eq!(store.present_ranges("reused").await.unwrap(), vec![0..4]);
    std::fs::remove_dir_all(root).unwrap();
}

fn binding(state: &mut DeliveryState, post: &str) -> RepresentationBinding {
    state
        .take_representation_bindings()
        .into_iter()
        .find(|binding| binding.post() == &PostId::new(post))
        .expect("candidate binding")
}

fn candidate(id: &str, discovered_at: u64) -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![url(id)],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
        renditions: Vec::new(),
        discovered_at,
    }
}

fn url(id: &str) -> String {
    format!("https://media.example/{id}.mp4")
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
