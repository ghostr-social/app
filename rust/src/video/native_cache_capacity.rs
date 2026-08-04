use crate::video::native_models::NativeVideoCacheKey;
use anyhow::Error;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use tokio::sync::Mutex;

#[derive(Debug)]
struct NativeCacheCapacityFailure {
    shortfall: u64,
}

impl Display for NativeCacheCapacityFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("native video cache budget exhausted")
    }
}

impl std::error::Error for NativeCacheCapacityFailure {}

pub fn capacity_exhausted(used: u64, requested: u64, maximum: u64) -> Error {
    let shortfall = used
        .checked_add(requested)
        .map_or(u64::MAX, |next| next.saturating_sub(maximum))
        .max(1);
    NativeCacheCapacityFailure { shortfall }.into()
}

#[derive(Default)]
pub struct NativeCacheCapacity {
    targets: Mutex<HashMap<NativeVideoCacheKey, u64>>,
}

impl NativeCacheCapacity {
    pub async fn remember(&self, key: &NativeVideoCacheKey, error: &Error, used: u64) {
        let Some(shortfall) = capacity_shortfall(error) else {
            return;
        };
        let target = used.saturating_sub(shortfall);
        self.targets
            .lock()
            .await
            .entry(key.clone())
            .and_modify(|current| *current = (*current).min(target))
            .or_insert(target);
    }

    pub async fn forget(&self, key: &NativeVideoCacheKey) {
        self.targets.lock().await.remove(key);
    }

    pub async fn retain(&self, active: &HashSet<NativeVideoCacheKey>) {
        self.targets
            .lock()
            .await
            .retain(|key, _| active.contains(key));
    }
}

fn capacity_shortfall(error: &Error) -> Option<u64> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<NativeCacheCapacityFailure>())
        .map(|failure| failure.shortfall)
}
