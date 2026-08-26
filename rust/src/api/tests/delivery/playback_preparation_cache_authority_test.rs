use crate::api::delivery_types::FfiPlaybackPreparationPlan;
use crate::api::playback_preparation_stream::{
    watch_preparation, PreparationContext, PreparationOut,
};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::command_channel;
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
async fn preparation_waits_for_exact_cache_authority() {
    let store = temp_store("ghostr-preparation-cache-authority");
    let tracked = TrackedItems::new();
    let mut meta = sized_meta(16, 2_000);
    meta.sha256 = Some("a".repeat(64));
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
    let cache = CacheRegistry::new();
    cache.insert("clip");
    let (handle, mut commands) = command_channel();
    commands.publish_focused_plan(7, Some(PostId::new("clip")), Default::default());
    let (sender, mut plans) = mpsc::unbounded_channel();
    let context = PreparationContext {
        endpoint: "127.0.0.1:8080".to_owned(),
        store,
        capabilities: ProgressiveCapabilities::production(),
        delivery: handle,
        tracked,
        cache: cache.clone(),
    };
    tokio::spawn(watch_preparation(ChannelOut(sender), context));

    assert!(recv(&mut plans).await.current.is_none());
    let mut mirror = meta.clone();
    mirror.urls = vec!["https://mirror.example/clip.mp4".to_owned()];
    cache.replace([video("clip", mirror)]);
    assert!(recv(&mut plans).await.current.is_some());
    let mut replacement = meta;
    replacement.sha256 = Some("b".repeat(64));
    cache.replace([video("clip", replacement)]);
    assert!(recv(&mut plans).await.current.is_none());
}

fn video(id: &str, meta: ghostr_engine::VideoMeta) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta,
        status: CacheStatus::Complete,
    }
}

async fn recv(
    plans: &mut mpsc::UnboundedReceiver<FfiPlaybackPreparationPlan>,
) -> FfiPlaybackPreparationPlan {
    tokio::time::timeout(Duration::from_secs(1), plans.recv())
        .await
        .expect("plan deadline")
        .expect("preparation plan")
}
