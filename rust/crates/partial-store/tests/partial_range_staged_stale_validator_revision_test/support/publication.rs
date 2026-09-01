use super::Fixture;
use crate::partial_range_store::ResponseOpenResult;
use ghostr_engine::adaptive::WholeBodyContract;

impl Fixture {
    pub(crate) async fn publish_full_body(&self, body: &[u8]) {
        let action = self
            .store
            .reserve_action(&self.transfer, 2, body.len() as u64)
            .await
            .expect("whole reservation");
        let opened = self
            .store
            .open_durable_single_response(
                &self.transfer,
                &action,
                WholeBodyContract::Exact {
                    expected_bytes: body.len() as u64,
                },
                self.lease.clone(),
            )
            .await
            .expect("whole open");
        assert_eq!(opened, ResponseOpenResult::Opened);
        assert!(self
            .store
            .write_single_response_for_action(&self.transfer, &action, 0, body)
            .await
            .expect("whole write"));
        assert!(self
            .store
            .finish_single_response_for_action(
                &self.transfer,
                &action,
                Some(body.len() as u64),
                true,
            )
            .await
            .expect("whole finish"));
        self.store.release_action(&action).await;
    }

    pub(crate) async fn stale_lease_rejected(&self, expected_bytes: usize) -> bool {
        let action = self
            .store
            .reserve_action(&self.transfer, 3, expected_bytes as u64)
            .await
            .expect("stale lease reservation");
        let opened = self
            .store
            .open_durable_single_response(
                &self.transfer,
                &action,
                WholeBodyContract::Exact {
                    expected_bytes: expected_bytes as u64,
                },
                self.lease.clone(),
            )
            .await
            .expect("stale lease open");
        self.store.release_action(&action).await;
        opened == ResponseOpenResult::Stale
    }
}
