use std::collections::HashSet;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

/// Shared registry of the post ids the progressive gateway may serve.
/// The delivery manager keeps it in sync with the catalog; requests for
/// unregistered ids are rejected with 404.
#[derive(Clone, Debug, Default)]
pub struct ServablePosts {
    inner: Arc<RwLock<HashSet<String>>>,
}

impl ServablePosts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, id: impl Into<String>) {
        self.write().insert(id.into());
    }

    pub fn remove(&self, id: &str) {
        self.write().remove(id);
    }

    /// Replaces the whole registry, e.g. with the current focus window.
    pub fn replace_all(&self, ids: impl IntoIterator<Item = String>) {
        let mut guard = self.write();
        guard.clear();
        guard.extend(ids);
    }

    pub fn contains(&self, id: &str) -> bool {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains(id)
    }

    fn write(&self) -> RwLockWriteGuard<'_, HashSet<String>> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
