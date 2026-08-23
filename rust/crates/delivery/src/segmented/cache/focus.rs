use super::{objects, FocusRecord, SegmentedCache, SegmentedPhase, SegmentedSnapshot};
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};

pub(crate) type PreservedFocus = HashMap<PostId, (u64, String)>;

impl SegmentedCache {
    #[cfg(test)]
    pub(crate) fn replace_focus(&self, generation: u64, items: Vec<(PostId, Vec<String>)>) {
        let protected = items.iter().map(|(post, _)| post.clone()).collect();
        self.replace_focus_window(generation, items, &protected);
    }

    #[cfg(test)]
    pub(crate) fn replace_focus_window(
        &self,
        generation: u64,
        items: Vec<(PostId, Vec<String>)>,
        protected: &HashSet<PostId>,
    ) {
        self.reconcile_focus_window(generation, items, protected, &HashMap::new());
    }

    pub(crate) fn reconcile_focus_window(
        &self,
        generation: u64,
        items: Vec<(PostId, Vec<String>)>,
        protected: &HashSet<PostId>,
        preserved: &PreservedFocus,
    ) {
        let mut state = self.lock();
        let mut next = HashMap::new();
        for (post, sources) in items {
            let previous = state.focus.remove(&post);
            let record = reconcile_record(
                previous,
                RecordInput {
                    generation,
                    sources,
                    protected,
                    preserved,
                    post: &post,
                },
            );
            next.insert(post, record);
        }
        state.focus = next;
        objects::retain_referenced(&mut state);
        drop(state);
        self.changed.notify_waiters();
    }

    pub(crate) fn root_source(&self, post: &PostId) -> Option<String> {
        self.lock().focus.get(post)?.root_source.clone()
    }

    pub(crate) fn focus_generation(&self, post: &PostId) -> Option<u64> {
        self.lock().focus.get(post).map(|record| record.generation)
    }
}

struct RecordInput<'a> {
    generation: u64,
    sources: Vec<String>,
    protected: &'a HashSet<PostId>,
    preserved: &'a PreservedFocus,
    post: &'a PostId,
}

fn reconcile_record(previous: Option<FocusRecord>, input: RecordInput<'_>) -> FocusRecord {
    let active = input.preserved.get(input.post);
    let reusable = previous.filter(|record| reusable(record, &input.sources, active));
    let mut record = reusable.unwrap_or_else(|| empty_record(input.generation));
    record.sources = input.sources;
    record.protected = input.protected.contains(input.post);
    record
}

fn reusable(record: &FocusRecord, sources: &[String], active: Option<&(u64, String)>) -> bool {
    let ready = record.snapshot.phase == SegmentedPhase::Ready
        && record
            .root_source
            .as_deref()
            .is_some_and(|root| super::super::source_key::contains(sources, root));
    let working = active.is_some_and(|(generation, root)| {
        record.generation == *generation && super::super::source_key::contains(sources, root)
    });
    ready || working
}

fn empty_record(generation: u64) -> FocusRecord {
    FocusRecord {
        generation,
        sources: Vec::new(),
        root_source: None,
        protected: false,
        snapshot: SegmentedSnapshot::default(),
        objects: Vec::new(),
        staged: Vec::new(),
        preparing: None,
        reserved_bytes: 0,
        assembly_bytes: 0,
    }
}
