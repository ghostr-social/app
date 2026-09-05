use super::player_preparation_authority_fixture::AuthorityFixture;
use crate::api::playback_preparation_stream::PreparationContext;
use crate::api::tests::support::{bind_store, sized_meta};

pub(super) async fn evict_capability(fixture: &AuthorityFixture) {
    let meta = sized_meta(16, 2_000);
    bind_store(&fixture.context.store, "other", &meta).await;
    fixture
        .context
        .store
        .set_total_len("other", 16)
        .await
        .expect("fixture");
    fixture
        .context
        .store
        .write_range("other", 0, &[3; 16])
        .await
        .expect("fixture");
    let snapshot = fixture
        .context
        .store
        .media_snapshot("other")
        .await
        .expect("fixture");
    fixture
        .context
        .capabilities
        .issue(&snapshot)
        .await
        .expect("fixture");
}

pub(super) fn context(fixture: &AuthorityFixture) -> PreparationContext {
    PreparationContext {
        endpoint: "127.0.0.1:8080".to_owned(),
        store: std::sync::Arc::clone(&fixture.context.store),
        capabilities: fixture.context.capabilities.clone(),
        delivery: fixture.context.delivery.clone(),
        tracked: fixture.context.tracked.clone(),
        cache: fixture.context.cache.clone(),
    }
}
