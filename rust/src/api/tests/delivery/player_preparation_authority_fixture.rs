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
        Self::seeded_with(ProgressiveCapabilities::production()).await
    }

    pub(super) async fn seeded_with(capabilities: ProgressiveCapabilities) -> Self {
        let store = temp_store("ghostr-player-preparation-authority");
        let tracked = TrackedItems::new();
        let meta = sized_meta(16, 2_000);
        bind_store(&store, "clip", &meta).await;
        store
            .set_total_len("clip", 16)
            .await
            .expect("test fixture precondition must hold");
        store
            .write_range("clip", 0, &[7; 16])
            .await
            .expect("test fixture precondition must hold");
        tracked.insert("clip".to_owned(), meta.clone());
        let snapshot = store
            .media_snapshot("clip")
            .await
            .expect("test fixture precondition must hold");
        let representation = snapshot
            .binding()
            .expect("test fixture precondition must hold")
            .representation()
            .fingerprint()
            .to_owned();
        let asset = capabilities
            .issue(&snapshot)
            .await
            .expect("test fixture precondition must hold")
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
            segmented: Default::default(),
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
            sequence: 1,
            state: FfiPlayerPreparationState::Initializing,
            failure_kind: None,
            observed_monotonic_us: 5,
        }
    }

    pub(super) async fn renew_content_revision(&self) -> String {
        self.context
            .store
            .evict_ranges("clip", core::slice::from_ref(&(8..16)))
            .await
            .expect("test fixture precondition must hold");
        let snapshot = self
            .context
            .store
            .media_snapshot("clip")
            .await
            .expect("test fixture precondition must hold");
        self.context
            .capabilities
            .issue(&snapshot)
            .await
            .expect("test fixture precondition must hold")
            .as_str()
            .to_owned()
    }
}
