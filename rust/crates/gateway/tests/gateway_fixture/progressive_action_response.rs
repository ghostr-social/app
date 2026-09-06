use super::progressive::{progressive_harness, ProgressiveHarness};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{DeliveryKind, VideoMeta};
use ghostr_partial_store::partial_range_store::{ResponseOpenResult, StoreAction};

const SOURCE: &str = "https://media.example/clip.mp4";

pub struct ActionResponse {
    pub gateway: ProgressiveHarness,
    pub action: StoreAction,
    identity: TransferIdentity,
}

impl ActionResponse {
    pub async fn new(total: u64) -> Self {
        let gateway = progressive_harness("ghostr-gateway-action-prefix");
        let meta = metadata(total);
        gateway.bind_video_meta("clip", meta.clone()).await;
        gateway.posts.replace([super::cache_video("clip", meta)]);
        let identity = gateway
            .store
            .representation_binding("clip")
            .await
            .expect("bound representation")
            .transfer(SOURCE)
            .expect("bound source");
        gateway
            .store
            .select_transfer(identity.clone())
            .await
            .expect("select response");
        let action = gateway
            .store
            .reserve_action(&identity, 1, total)
            .await
            .expect("reserve response");
        assert_eq!(
            gateway
                .store
                .open_action_scoped_single_response(
                    &identity,
                    &action,
                    WholeBodyContract::Exact {
                        expected_bytes: total
                    },
                )
                .await
                .expect("open response"),
            ResponseOpenResult::Opened,
            "the gateway fixture must own the active response"
        );
        Self {
            gateway,
            action,
            identity,
        }
    }

    pub async fn write(&self, offset: u64, bytes: &[u8]) {
        assert!(
            self.gateway
                .store
                .write_single_response_for_action(&self.identity, &self.action, offset, bytes)
                .await
                .expect("write active response"),
            "the active response must accept the contiguous bytes"
        );
    }
}

fn metadata(total: u64) -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(total),
        duration_ms: Some(1_000),
    }
}
