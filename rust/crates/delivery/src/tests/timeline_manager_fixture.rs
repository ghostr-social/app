use crate::delivery_events::{command_channel, DeliveryFocus, DeliveryHandle, FocusItem};
use crate::manager::timeline::TimelineParser;
use crate::manager::DeliveryWorker;
use crate::playback_demand::demand_channel;
use crate::tests::timeline_manager_environment;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::media_timeline::MediaTimeline;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct TimelineManagerFixture {
    pub(crate) root: PathBuf,
    pub(crate) store: Arc<PartialRangeStore>,
    pub(crate) handle: DeliveryHandle,
    pub(crate) worker: DeliveryWorker,
    pub(crate) post: PostId,
    meta: VideoMeta,
}

impl TimelineManagerFixture {
    pub(crate) async fn new(parser: Arc<dyn TimelineParser>) -> Self {
        let root = super::support::temp_directory("timeline-manager");
        let store = Arc::new(PartialRangeStore::with_capacity(
            root.clone(),
            Arc::new(Mutex::new(0)),
            StoreCapacity::system(u64::MAX),
        ));
        let post = PostId::new("post");
        let meta = media_meta();
        let mut catalog = Catalog::new();
        let binding = catalog.upsert(post.clone(), meta.clone());
        store.bind_representation(binding).await.unwrap();
        store
            .write_range(post.as_str(), 0, &[0; 512])
            .await
            .unwrap();
        store.set_total_len(post.as_str(), 2_048).await.unwrap();
        let (handle, commands) = command_channel();
        let (_demand, demand) = demand_channel();
        let config = timeline_manager_environment::config(store.clone(), root.clone());
        let worker =
            DeliveryWorker::create_with_timeline_parser(config, commands, demand, parser).await;
        Self {
            root,
            store,
            handle,
            worker,
            post,
            meta,
        }
    }

    pub(crate) fn focus(&self) {
        self.handle.update_focus(DeliveryFocus::compatibility(
            vec![FocusItem {
                post: self.post.clone(),
                meta: self.meta.clone(),
            }],
            0,
            0,
        ));
    }

    pub(crate) fn timeline(&self) -> Option<MediaTimeline> {
        self.worker.timeline_for_test(&self.post)
    }
}

fn media_meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://unused.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(2_048),
        duration_ms: Some(1_000),
    }
}
