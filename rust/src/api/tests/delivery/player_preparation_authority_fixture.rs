use crate::api::delivery_types::{FfiPlayerPreparationReport, FfiPlayerPreparationState};
use crate::api::player_preparation_control::PlayerPreparationContext;
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::{command_channel, CommandReceiver};
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;

pub(super) struct AuthorityFixture {
    pub(super) context: PlayerPreparationContext,
    pub(super) representation: String,
    pub(super) asset: String,
    pub(super) commands: CommandReceiver,
}

impl AuthorityFixture {
    pub(super) async fn seeded() -> Self {
        let store = temp_store("ghostr-player-preparation-authority");
        let tracked = TrackedItems::new();
        let meta = sized_meta(16, 2_000);
        bind_store(&store, "clip", &meta).await;
        store.set_total_len("clip", 16).await.unwrap();
        store.write_range("clip", 0, &[7; 16]).await.unwrap();
        tracked.insert("clip".to_owned(), meta.clone());
        let snapshot = store.media_snapshot("clip").await.unwrap();
        let representation = snapshot
            .binding()
            .unwrap()
            .representation()
            .fingerprint()
            .to_owned();
        let capabilities = ProgressiveCapabilities::production();
        let asset = capabilities
            .issue(&snapshot)
            .await
            .unwrap()
            .as_str()
            .to_owned();
        let cache = CacheRegistry::new();
        cache.replace([CacheVideo {
            id: "clip".to_owned(),
            meta,
            status: CacheStatus::Complete,
        }]);
        let (delivery, commands) = command_channel();
        let context = PlayerPreparationContext {
            store,
            capabilities,
            delivery,
            tracked,
            cache,
        };
        Self {
            context,
            representation,
            asset,
            commands,
        }
    }

    pub(super) fn input(&self) -> FfiPlayerPreparationReport {
        FfiPlayerPreparationReport {
            post_id: "clip".to_owned(),
            representation_id: self.representation.clone(),
            asset_id: self.asset.clone(),
            player_capability_generation: 1,
            client_epoch: 2,
            attempt_generation: 3,
            sequence: 4,
            state: FfiPlayerPreparationState::Initialized,
            failure_kind: None,
            observed_monotonic_us: 5,
        }
    }
}
