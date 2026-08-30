use crate::segmented::cache::{PreservedFocus, SegmentedFocusItem};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::{CachedHlsGeneration, HlsPreparedAssetAuthority, SegmentedCache};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationId;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use url::Url;

pub(super) struct PreparedAuthorityFixture {
    pub cache: SegmentedCache,
    pub post: PostId,
    pub representation: RepresentationId,
    root: String,
    protected: bool,
}

impl PreparedAuthorityFixture {
    pub fn new(root: &str, protected: bool) -> Self {
        let mut fixture = Self {
            cache: SegmentedCache::new(),
            post: PostId::new("post"),
            representation: representation(root),
            root: root.to_owned(),
            protected,
        };
        fixture.replace_focus(1, root, protected);
        fixture
    }

    pub fn replace_focus(&mut self, generation: u64, root: &str, protected: bool) {
        self.root = root.to_owned();
        self.protected = protected;
        self.representation = representation(root);
        let protected = protected.then(|| self.post.clone()).into_iter().collect();
        let item = SegmentedFocusItem::new(
            self.post.clone(),
            self.representation.clone(),
            vec![self.root.clone()],
        );
        self.cache.reconcile_focus_window(
            generation,
            vec![item],
            &protected,
            &PreservedFocus::new(),
        );
    }

    pub fn publish(&self, generation: u64, body: &[u8]) -> CachedHlsGeneration {
        let object = object(&self.root, body);
        assert!(self.cache.mark_stage_preparing(
            &self.post,
            generation,
            1,
            object.body.len() as u64,
        ));
        assert!(self
            .cache
            .store_stage_object(&self.post, generation, object)
            .is_some());
        assert!(self.cache.mark_stage_ready(&self.post, generation));
        self.cache
            .object(&self.root)
            .expect("published object")
            .generation()
    }

    pub fn authority(&self) -> HlsPreparedAssetAuthority {
        self.cache
            .snapshot(self.post.as_str())
            .authority
            .expect("ready authority")
    }
}

fn representation(root: &str) -> RepresentationId {
    let post = PostId::new("post");
    let meta = VideoMeta {
        urls: vec![root.to_owned()],
        delivery: DeliveryKind::Hls,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    };
    Catalog::new().upsert(post, meta).representation().clone()
}

fn object(root: &str, body: &[u8]) -> PreparedObject {
    PreparedObject {
        request_url: root.to_owned(),
        final_url: Url::parse(root).expect("valid test URL"),
        body: Arc::from(body),
        content_type: Some("application/vnd.apple.mpegurl".to_owned()),
        cache: Default::default(),
    }
}
