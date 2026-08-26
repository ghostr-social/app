use crate::api::delivery_types::{FfiPlaybackPreparationPlan, FfiPlaybackPreparationReadiness};
use crate::api::playback_preparation_stream::{
    watch_preparation, PreparationContext, PreparationOut,
};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::delivery::playback_preparation_sparse_fixture::complete_startup;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::command_channel;
use ghostr_delivery::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{AllocationPlan, NextReserveEvidence};
use ghostr_engine::PostId;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use tokio::sync::mpsc;

struct ChannelOut(mpsc::UnboundedSender<FfiPlaybackPreparationPlan>);

impl PreparationOut for ChannelOut {
    fn send(&self, plan: FfiPlaybackPreparationPlan) -> bool {
        self.0.send(plan).is_ok()
    }
}

#[tokio::test]
async fn projects_exact_current_and_structurally_ready_adjacent_next_assets() {
    let store = temp_store("ghostr-preparation-plan");
    let tracked = TrackedItems::new();
    prepare(&store, &tracked, "current").await;
    prepare(&store, &tracked, "next").await;
    let startup = complete_startup(&sized_meta(16, 2_000), 16);
    let certificate = StartupCertificate::issue(
        startup.clone(),
        &store
            .media_snapshot("next")
            .await
            .expect("test fixture precondition must hold"),
    )
    .expect("test fixture precondition must hold");
    let (handle, mut commands) = command_channel();
    let plan = AllocationPlan {
        next_reserve: NextReserveEvidence::Ready {
            post: PostId::new("next"),
            startup,
        },
        ..AllocationPlan::default()
    };
    commands.publish_focused_plan_with_startup(
        42,
        Some(PostId::new("current")),
        plan,
        Some(certificate),
    );
    let capabilities = ProgressiveCapabilities::production();
    let cache = CacheRegistry::new();
    cache.replace([cached("current"), cached("next")]);
    let (sender, mut plans) = mpsc::unbounded_channel();
    let context = PreparationContext {
        endpoint: "127.0.0.1:8080".to_owned(),
        store,
        capabilities,
        delivery: handle,
        tracked,
        cache,
    };
    tokio::spawn(watch_preparation(ChannelOut(sender), context));

    let plan = tokio::time::timeout(Duration::from_secs(1), plans.recv())
        .await
        .expect("plan deadline")
        .expect("preparation plan");
    assert_eq!(plan.current_delivery_id.as_deref(), Some("current"));
    let current = plan.current.expect("current asset");
    let next = plan.next.expect("next asset");
    assert_eq!(plan.revision, 1);
    assert_eq!(current.delivery_id, "current");
    assert_eq!(next.delivery_id, "next");
    assert_eq!(current.representation_id.len(), 64);
    assert_eq!(next.representation_id.len(), 64);
    assert_eq!(current.representation_id, next.representation_id);
    assert_ne!(current.asset_id, next.asset_id);
    assert_eq!(next.readiness, FfiPlaybackPreparationReadiness::Ready);
}

fn cached(id: &str) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta: sized_meta(16, 2_000),
        status: CacheStatus::Complete,
    }
}

async fn prepare(
    store: &ghostr_partial_store::partial_range_store::PartialRangeStore,
    tracked: &TrackedItems,
    id: &str,
) {
    let meta = sized_meta(16, 2_000);
    bind_store(store, id, &meta).await;
    store
        .set_total_len(id, 16)
        .await
        .expect("test fixture precondition must hold");
    store
        .write_range(id, 0, &[7; 16])
        .await
        .expect("test fixture precondition must hold");
    tracked.insert(id.to_owned(), meta);
}
