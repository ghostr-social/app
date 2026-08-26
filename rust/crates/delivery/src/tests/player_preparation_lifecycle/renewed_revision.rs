use crate::client_capability::ClientCapabilityStatus;
use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_fixture::{evidence, state, EvidenceSpec};
use crate::tests::support::temp_directory;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn renewed_revision_keeps_live_decode_learning_without_inheriting_readiness() {
    let mut state = state(&["current", "next"], 0);
    let post = PostId::new("next");
    let root = temp_directory("player-preparation-renewed-revision");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    store
        .bind_representation(state.catalog().binding(&post).expect("valid test fixture"))
        .await
        .expect("valid test fixture");
    store.set_total_len(post.as_str(), 16).await.expect("valid test fixture");
    store.write_range(post.as_str(), 0, &[7; 16]).await.expect("valid test fixture");
    let original = store.media_snapshot(post.as_str()).await.expect("valid test fixture").revision();
    apply(&mut state, original, PlayerPreparationState::Initializing);
    let removed = 8..16;
    store
        .evict_ranges(post.as_str(), core::slice::from_ref(&removed))
        .await
        .expect("valid test fixture");
    let renewed = store.media_snapshot(post.as_str()).await.expect("valid test fixture").revision();

    assert_ne!(renewed, original);
    assert_eq!(
        state.player_preparation(&post, Some(renewed)),
        ghostr_engine::adaptive::PlayerPreparation::Unverified,
    );
    state.prune_player_preparations(&HashMap::from([(post.clone(), renewed)]));
    assert_eq!(
        state.client_capability_status(&post, 1, 1),
        ClientCapabilityStatus::Testing,
    );
    apply(
        &mut state,
        original,
        PlayerPreparationState::FirstFrameRendered,
    );
    assert_eq!(
        state.client_capability_status(&post, 1, 1),
        ClientCapabilityStatus::Supported {
            p95_first_frame_us: 1,
        },
    );
    assert_eq!(
        state.player_preparation(&post, Some(renewed)),
        ghostr_engine::adaptive::PlayerPreparation::Unverified,
    );
    tokio::fs::remove_dir_all(root).await.expect("valid test fixture");
}

fn apply(
    state: &mut crate::manager::state::DeliveryState,
    revision: ghostr_partial_store::partial_range_store::ContentRevision,
    preparation: PlayerPreparationState,
) {
    let sequence = match preparation {
        PlayerPreparationState::Initializing => 1,
        _ => 2,
    };
    let report = evidence(
        state,
        EvidenceSpec {
            post: "next",
            revision,
            sequence,
            state: preparation,
        },
    );
    assert!(state.apply_player_preparation(report));
}
