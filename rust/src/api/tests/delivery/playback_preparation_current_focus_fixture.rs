use super::playback_preparation_current_lifecycle_fixture::CurrentLifecycleFixture;
use crate::api::tests::support::bind_store;
use ghostr_delivery::cache_registry::{CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_engine::{PostId, VideoMeta};

pub(super) async fn seed_other(fixture: &CurrentLifecycleFixture, meta: &VideoMeta) {
    let context = &fixture.manager.context;
    bind_store(&context.store, "other", meta).await;
    context
        .store
        .set_total_len("other", 16)
        .await
        .expect("test fixture precondition must hold");
    context
        .store
        .write_range("other", 0, &[8; 16])
        .await
        .expect("test fixture precondition must hold");
    context.tracked.insert("other".to_owned(), meta.clone());
    context
        .cache
        .replace([cached("clip", meta), cached("other", meta)]);
}

pub(super) fn focus_other(meta: &VideoMeta) -> DeliveryFocus {
    DeliveryFocus::compatibility(vec![item("clip", meta), item("other", meta)], 1, 0)
}

fn item(id: &str, meta: &VideoMeta) -> FocusItem {
    FocusItem {
        post: PostId::new(id),
        meta: meta.clone(),
    }
}

fn cached(id: &str, meta: &VideoMeta) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta: meta.clone(),
        status: CacheStatus::Complete,
    }
}
