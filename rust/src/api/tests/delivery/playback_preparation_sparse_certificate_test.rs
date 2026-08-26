use crate::api::delivery_types::FfiPlaybackPreparationPlan;
use crate::api::playback_preparation_stream::{
    watch_preparation, PreparationContext, PreparationOut,
};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::delivery::playback_preparation_sparse_fixture::sparse_startup;
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
async fn exact_sparse_startup_certificate_projects_the_adjacent_asset() {
    let store = temp_store("ghostr-sparse-preparation");
    let tracked = TrackedItems::new();
    seed_current(&store, &tracked).await;
    let sparse = sparse_startup();
    bind_store(&store, "next", &sparse.meta).await;
    store
        .set_total_len("next", sparse.total)
        .await
        .expect("test fixture precondition must hold");
    for (offset, bytes) in &sparse.writes {
        store
            .write_range("next", *offset, bytes)
            .await
            .expect("test fixture precondition must hold");
    }
    tracked.insert("next".to_owned(), sparse.meta.clone());
    let startup = sparse.startup;
    let snapshot = store
        .media_snapshot("next")
        .await
        .expect("test fixture precondition must hold");
    assert!(!snapshot.is_complete());
    let certificate = StartupCertificate::issue(startup.clone(), &snapshot)
        .expect("test fixture precondition must hold");
    let plan = AllocationPlan {
        next_reserve: NextReserveEvidence::Structural {
            post: PostId::new("next"),
            startup,
        },
        ..Default::default()
    };
    let (delivery, mut commands) = command_channel();
    commands.publish_focused_plan_with_startup(
        7,
        Some(PostId::new("current")),
        plan,
        Some(certificate),
    );
    let cache = CacheRegistry::new();
    cache.replace([
        cached("current", sized_meta(16, 2_000)),
        cached("next", sparse.meta.clone()),
    ]);
    let (sender, mut plans) = mpsc::unbounded_channel();
    tokio::spawn(watch_preparation(
        ChannelOut(sender),
        PreparationContext {
            endpoint: "127.0.0.1:8080".to_owned(),
            store,
            capabilities: ProgressiveCapabilities::production(),
            delivery,
            tracked,
            cache,
        },
    ));
    let projected = tokio::time::timeout(Duration::from_secs(1), plans.recv())
        .await
        .expect("test fixture precondition must hold")
        .expect("test fixture precondition must hold");
    assert_eq!(
        projected
            .next
            .expect("test fixture precondition must hold")
            .delivery_id,
        "next"
    );
}

async fn seed_current(
    store: &ghostr_partial_store::partial_range_store::PartialRangeStore,
    tracked: &TrackedItems,
) {
    let meta = sized_meta(16, 2_000);
    bind_store(store, "current", &meta).await;
    store
        .set_total_len("current", 16)
        .await
        .expect("test fixture precondition must hold");
    store
        .write_range("current", 0, &[7; 16])
        .await
        .expect("test fixture precondition must hold");
    tracked.insert("current".to_owned(), meta);
}

fn cached(id: &str, meta: ghostr_engine::VideoMeta) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta,
        status: CacheStatus::Partial,
    }
}
