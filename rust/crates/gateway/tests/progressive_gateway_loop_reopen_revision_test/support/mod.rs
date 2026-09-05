use crate::gateway_fixture::progressive::ProgressiveHarness;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{HttpGenerationLease, TransferIdentity};
use ghostr_partial_store::partial_range_store::{
    ResponseOpenResult, StoreAction, StoredMediaSnapshot,
};

mod setup;
pub use setup::{seeded_harness, serve};

pub const BODY: &[u8] = b"abcd";

pub struct LoopReopenFixture {
    pub harness: ProgressiveHarness,
    pub(super) transfer: TransferIdentity,
    pub(super) generation: HttpGenerationLease,
}

impl LoopReopenFixture {
    pub fn cleanup(self) {
        let root = self.harness.root.clone();
        drop(self);
        std::fs::remove_dir_all(root).expect("remove store");
    }

    pub async fn snapshot(&self) -> StoredMediaSnapshot {
        self.harness
            .store
            .media_snapshot("clip")
            .await
            .expect("snapshot")
    }

    pub async fn commit_durable_full_body(&self) {
        let action = self.open_durable_response().await;
        self.write_durable_response(&action).await;
        self.finish_durable_response(&action).await;
        self.harness.store.release_action(&action).await;
    }

    async fn open_durable_response(&self) -> StoreAction {
        let action = self
            .harness
            .store
            .reserve_action(&self.transfer, 7, BODY.len() as u64)
            .await
            .expect("reserve");
        let contract = WholeBodyContract::Exact {
            expected_bytes: BODY.len() as u64,
        };
        let opened = self
            .harness
            .store
            .open_durable_single_response(
                &self.transfer,
                &action,
                contract,
                self.generation.clone(),
            )
            .await
            .expect("full response open");
        assert_eq!(
            opened,
            ResponseOpenResult::Opened,
            "replacement response opens"
        );
        action
    }

    async fn write_durable_response(&self, action: &StoreAction) {
        self.harness
            .store
            .write_single_response_for_action(&self.transfer, action, 0, BODY)
            .await
            .expect("full response bytes");
    }

    async fn finish_durable_response(&self, action: &StoreAction) {
        let finished = self
            .harness
            .store
            .finish_single_response_for_action(
                &self.transfer,
                action,
                Some(BODY.len() as u64),
                true,
            )
            .await
            .expect("full response finish");
        assert!(finished, "replacement finishes");
    }
}
