use crate::api::delivery_types::{
    FfiPlayerPreparationDisposition, FfiPlayerPreparationReport, FfiPlayerPreparationState,
};
use crate::api::player_preparation_control::{
    confirm_player_preparation, PlayerPreparationContext,
};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::hls_runtime_support;

#[tokio::test]
async fn hls_feedback_requires_the_exact_live_segmented_asset_revision() {
    let (runtime, snapshot, root, meta) = hls_runtime_support::prepared_with_meta().await;
    let authority = snapshot.authority.expect("prepared HLS authority");
    let progressive = runtime.progressive();
    let tracked = TrackedItems::new();
    tracked.insert("stream".to_owned(), meta);
    let context = PlayerPreparationContext {
        store: progressive.store.clone(),
        capabilities: progressive.capabilities.clone(),
        delivery: runtime.delivery(),
        tracked,
        cache: progressive.cache.clone(),
        segmented: runtime.segmented(),
    };
    assert_invalid_tokens(&context, &authority).await;
    let input = report(&authority, 10);

    assert_eq!(
        confirm_player_preparation(&context, input.clone()).await,
        FfiPlayerPreparationDisposition::Applied,
    );
    runtime.segmented().clear();
    assert_eq!(
        confirm_player_preparation(&context, report(&authority, 11)).await,
        FfiPlayerPreparationDisposition::Rejected,
    );

    drop(input);
    drop(context);
    drop(runtime);
    std::fs::remove_dir_all(root).ok();
}

async fn assert_invalid_tokens(
    context: &PlayerPreparationContext,
    authority: &ghostr_delivery::segmented::HlsPreparedAssetAuthority,
) {
    let tokens = [
        "hls-v1:0",
        "hls-v1:01",
        "1",
        "hls-v1:one",
        "hls-v1:18446744073709551616",
    ];
    for (index, token) in tokens.into_iter().enumerate() {
        let mut invalid = report(authority, index as u64 + 1);
        invalid.asset_id = token.to_owned();
        assert_eq!(
            confirm_player_preparation(context, invalid).await,
            FfiPlayerPreparationDisposition::Rejected,
        );
    }
}

fn report(
    authority: &ghostr_delivery::segmented::HlsPreparedAssetAuthority,
    attempt_generation: u64,
) -> FfiPlayerPreparationReport {
    FfiPlayerPreparationReport {
        post_id: authority.post().as_str().to_owned(),
        representation_id: authority.representation_id().fingerprint().to_owned(),
        asset_id: format!("hls-v1:{}", authority.asset_revision().value()),
        player_capability_generation: 1,
        client_epoch: 2,
        attempt_generation,
        sequence: 1,
        state: FfiPlayerPreparationState::Initializing,
        failure_kind: None,
        observed_monotonic_us: attempt_generation,
    }
}
