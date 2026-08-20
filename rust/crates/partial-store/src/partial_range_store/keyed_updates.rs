use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex, Weak};
use tokio::sync::{Mutex, OwnedMutexGuard, RwLockReadGuard};

#[derive(Default)]
pub(super) struct KeyedUpdates {
    locks: StdMutex<HashMap<String, Weak<Mutex<()>>>>,
}

impl KeyedUpdates {
    pub(super) async fn lock(&self, key: &str) -> OwnedMutexGuard<()> {
        let lock = {
            let mut locks = self.locks.lock().unwrap_or_else(|error| error.into_inner());
            locks.retain(|_, known| known.strong_count() > 0);
            if let Some(lock) = locks.get(key).and_then(Weak::upgrade) {
                lock
            } else {
                let lock = Arc::new(Mutex::new(()));
                locks.insert(key.to_owned(), Arc::downgrade(&lock));
                lock
            }
        };
        lock.lock_owned().await
    }
}

pub(super) struct UpdateGuard<'a> {
    _lifecycle: RwLockReadGuard<'a, ()>,
    _key: OwnedMutexGuard<()>,
}

impl super::PartialRangeStore {
    pub(super) async fn update_key(&self, key: &str) -> anyhow::Result<UpdateGuard<'_>> {
        let guard = self.observe_key(key).await?;
        anyhow::ensure!(
            self.policy_transaction_debt(key).await.is_none(),
            "policy transaction cleanup is still active"
        );
        Ok(guard)
    }

    pub(super) async fn observe_key(&self, key: &str) -> anyhow::Result<UpdateGuard<'_>> {
        let guard = self.update_key_raw(key).await;
        Box::pin(self.recover_policy_transaction_locked(key)).await?;
        Ok(guard)
    }

    pub(super) async fn update_key_raw(&self, key: &str) -> UpdateGuard<'_> {
        let lifecycle = self.representation_updates.read().await;
        let key = self.keyed_updates.lock(key).await;
        UpdateGuard {
            _lifecycle: lifecycle,
            _key: key,
        }
    }
}
