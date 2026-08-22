use crate::chunk::cancel::CancelToken;
use crate::manager::inflight::{ChunkAttempt, InFlightChunks, PromotionTarget};
use ghostr_engine::adaptive::PromotionGrant;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, StoreAction};
use std::path::PathBuf;

mod response;
mod setup;
pub(super) use response::response;
use setup::{registered, store_setup};

pub(crate) struct PromotionFixture {
    pub active: InFlightChunks,
    pub attempt: ChunkAttempt,
    pub action: StoreAction,
    pub target: PromotionTarget,
    pub token: CancelToken,
    pub store: PartialRangeStore,
    root: PathBuf,
}

impl PromotionFixture {
    pub async fn new(valid_until_ms: u64) -> Self {
        let setup = store_setup().await;
        let grant = PromotionGrant {
            maximum_bytes: 16,
            valid_until_ms,
        };
        let (active, attempt, token) = registered(&setup.identity, &setup.action, grant);
        Self {
            active,
            attempt,
            action: setup.action,
            target: PromotionTarget::new(ghostr_engine::ActionId::new(1), setup.identity, grant),
            token,
            store: setup.store,
            root: setup.root,
        }
    }

    pub async fn cleanup(self) {
        self.store.release_action(&self.action).await;
        drop(self.store);
        std::fs::remove_dir_all(self.root).unwrap();
    }
}
