use crate::api::delivery_types::{
    FfiPlayerPreparationDisposition, FfiPlayerPreparationReport, FfiPlayerPreparationState,
};
use crate::api::player_preparation_control::{
    confirm_player_preparation, PlayerPreparationContext,
};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::sized_meta;
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::delivery_events::command_channel;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[tokio::test]
async fn store_validation_failure_is_proven_not_admitted() {
    let root = std::env::temp_dir().join(format!(
        "ghostr-player-validation-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test fixture precondition must hold")
            .as_nanos()
    ));
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    std::fs::create_dir_all(root.join("clip.transform.video"))
        .expect("test fixture precondition must hold");
    let tracked = TrackedItems::new();
    tracked.insert("clip".to_owned(), sized_meta(16, 2_000));
    let (delivery, commands) = command_channel();
    let context = PlayerPreparationContext {
        store,
        capabilities: ProgressiveCapabilities::production(),
        delivery,
        tracked,
        cache: CacheRegistry::new(),
        segmented: Default::default(),
    };

    assert_eq!(
        confirm_player_preparation(&context, input()).await,
        FfiPlayerPreparationDisposition::NotAdmitted,
    );
    assert!(commands.try_player_preparation().is_none());
    drop(context);
    std::fs::remove_dir_all(root).expect("test fixture precondition must hold");
}

fn input() -> FfiPlayerPreparationReport {
    FfiPlayerPreparationReport {
        post_id: "clip".to_owned(),
        representation_id: "a".repeat(64),
        asset_id: "asset".to_owned(),
        player_capability_generation: 1,
        client_epoch: 2,
        attempt_generation: 3,
        sequence: 1,
        state: FfiPlayerPreparationState::Initializing,
        failure_kind: None,
        observed_monotonic_us: 5,
    }
}
