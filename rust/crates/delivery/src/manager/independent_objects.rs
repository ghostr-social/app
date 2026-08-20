use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ObjectSource {
    post: PostId,
    source: String,
}

#[derive(Default)]
pub(crate) struct IndependentObjects {
    required_at: HashMap<ObjectSource, ContentRevision>,
}

impl IndependentObjects {
    pub(crate) fn record(&mut self, post: PostId, source: String, revision: ContentRevision) {
        self.required_at
            .insert(ObjectSource { post, source }, revision);
    }

    pub(crate) fn current(
        &mut self,
        revisions: &HashMap<PostId, ContentRevision>,
    ) -> HashMap<PostId, HashSet<String>> {
        self.required_at
            .retain(|key, revision| revisions.get(&key.post) == Some(revision));
        let mut current = HashMap::<PostId, HashSet<String>>::new();
        for key in self.required_at.keys() {
            current
                .entry(key.post.clone())
                .or_default()
                .insert(key.source.clone());
        }
        current
    }

    pub(crate) fn clear(&mut self) {
        self.required_at.clear();
    }
}
