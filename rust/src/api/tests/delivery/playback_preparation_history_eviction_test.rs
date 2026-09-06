use super::playback_preparation_history_fixture::{focused_history, previous_asset};

#[tokio::test]
async fn evicted_previous_bytes_cannot_reuse_a_preparation_certificate() {
    let fixture = focused_history().await;
    assert!(previous_asset(&fixture).await.is_some());
    fixture
        .manager
        .context
        .store
        .clear()
        .await
        .expect("evict fixture bytes");
    let previous = previous_asset(&fixture).await;
    fixture.shutdown().await;
    assert!(
        previous.is_none(),
        "stale startup evidence must not authorize a preview"
    );
}
