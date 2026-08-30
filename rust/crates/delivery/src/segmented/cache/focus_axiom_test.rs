use super::*;

impl SegmentedCache {
    pub(crate) fn replace_focus(&self, generation: u64, items: Vec<(PostId, Vec<String>)>) {
        let protected = items.iter().map(|(post, _)| post.clone()).collect();
        self.replace_focus_window(generation, items, &protected);
    }
    pub(in super::super) fn replace_focus_window(
        &self,
        generation: u64,
        items: Vec<(PostId, Vec<String>)>,
        protected: &HashSet<PostId>,
    ) {
        let items = items.into_iter().map(test_focus_item).collect();
        self.reconcile_focus_window(generation, items, protected, &HashMap::new());
    }
}

fn test_focus_item((post, sources): (PostId, Vec<String>)) -> SegmentedFocusItem {
    let meta = ghostr_engine::VideoMeta {
        urls: sources.clone(),
        delivery: ghostr_engine::DeliveryKind::Hls,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    };
    SegmentedFocusItem::new(
        post,
        ghostr_engine::representation::RepresentationId::for_meta(&meta),
        sources,
    )
}
