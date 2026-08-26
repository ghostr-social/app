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
        self.reconcile_focus_window(generation, items, protected, &HashMap::new());
    }
}
