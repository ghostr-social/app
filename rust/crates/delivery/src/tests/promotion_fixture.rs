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
    pub(super) active: InFlightChunks,
    pub(super) attempt: ChunkAttempt,
    pub(super) action: StoreAction,
    pub(super) target: PromotionTarget,
    pub(super) token: CancelToken,
    pub(super) store: PartialRangeStore,
    root: PathBuf,
}

impl PromotionFixture {
    pub(super) async fn new(valid_until_ms: u64) -> Self {
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

    pub(super) async fn cleanup(self) {
        self.store.release_action(&self.action).await;
        drop(self.store);
        std::fs::remove_dir_all(self.root).expect("valid test fixture");
    }

    pub(super) fn observe_headers(&mut self, observed_at_ms: u64) {
        assert!(self
            .active
            .observe_headers(&self.attempt, &response(), observed_at_ms));
    }
}
