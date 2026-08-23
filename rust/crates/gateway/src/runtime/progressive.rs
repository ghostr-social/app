use crate::progressive::capabilities::ProgressiveCapabilityId;
use crate::progressive::route::ProgressiveState;
use anyhow::Context;
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::VideoMeta;
use tokio::time::{timeout_at, Instant};

pub(super) async fn issue(
    state: &ProgressiveState,
    post: &str,
    expected: &VideoMeta,
) -> anyhow::Result<ProgressiveCapabilityId> {
    let deadline = Instant::now() + state.timing.unknown_length_wait;
    let store_changes = state.store.change_notifier();
    let cache_changes = state.cache.notifier();
    loop {
        let store_wake = store_changes.notified();
        let cache_wake = cache_changes.notified();
        tokio::pin!(store_wake, cache_wake);
        store_wake.as_mut().enable();
        cache_wake.as_mut().enable();
        let snapshot = state.store.media_snapshot(post).await?;
        if snapshot
            .binding()
            .is_some_and(|binding| binding_is_current(&state.cache, post, expected, binding))
        {
            return state.capabilities.issue(&snapshot).await;
        }
        let changed = async {
            tokio::select! { _ = store_wake => {}, _ = cache_wake => {} }
        };
        timeout_at(deadline, changed)
            .await
            .context("progressive representation is not current")?;
    }
}

fn binding_is_current(
    cache: &CacheRegistry,
    post: &str,
    expected: &VideoMeta,
    binding: &RepresentationBinding,
) -> bool {
    binding.matches_source_meta(expected) && cache.matches_binding(post, binding)
}

#[cfg(test)]
mod tests;
