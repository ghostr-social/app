use super::progress::Pending;
use super::SegmentedDelivery;
use crate::segmented::SegmentedPhase;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::PostId;

impl SegmentedDelivery {
    pub(crate) fn root_sets(&self) -> Vec<(PostId, Vec<String>)> {
        self.targets
            .iter()
            .map(|target| (target.post.clone(), target.sources.clone()))
            .collect()
    }

    pub(crate) fn select_pending_root(&mut self, post: &PostId, root: &str) -> bool {
        let Some((index, _)) = self.root_entry(post, root) else {
            return false;
        };
        let Some(pending) = self.pending.get(post) else {
            return false;
        };
        if pending.stage != HlsBootstrapStage::RootManifest {
            return false;
        }
        let generation = pending.generation;
        if crate::segmented::source_key::canonical(&pending.root_source)
            == crate::segmented::source_key::canonical(root)
        {
            self.pending.get_mut(post).unwrap().source_index = index;
            return true;
        }
        if !self.cache.reset_stage_retry(post, generation) {
            return false;
        }
        let attempt = self.allocate_attempt();
        self.pending.insert(
            post.clone(),
            Pending::root(generation, attempt, index, root.to_owned()),
        );
        true
    }

    pub(crate) fn restart_pending_root(&mut self, post: &PostId, root: &str) -> bool {
        let Some((index, _)) = self.root_entry(post, root) else {
            return false;
        };
        let Some(pending) = self.pending.get(post) else {
            return false;
        };
        if crate::segmented::source_key::canonical(&pending.root_source)
            == crate::segmented::source_key::canonical(root)
        {
            return false;
        }
        let generation = pending.generation;
        if !self.cache.reset_stage_retry(post, generation) {
            return false;
        }
        let attempt = self.allocate_attempt();
        self.pending.insert(
            post.clone(),
            Pending::root(generation, attempt, index, root.to_owned()),
        );
        true
    }

    pub(crate) fn suspend_pending_roots(&mut self, post: &PostId) -> bool {
        let Some(pending) = self.pending.remove(post) else {
            return false;
        };
        let suspended = pending.stage == HlsBootstrapStage::RootManifest
            && self.cache.mark_stage_failed(
                post,
                pending.generation,
                "HLS sources are cooling".to_owned(),
            );
        if !suspended {
            self.pending.insert(post.clone(), pending);
        }
        suspended
    }

    pub(super) fn revive_root(&mut self, post: &PostId, root: String) -> bool {
        let Some((index, _)) = self.root_entry(post, &root) else {
            return false;
        };
        if self.cache.snapshot(post.as_str()).phase != SegmentedPhase::Failed {
            return false;
        }
        let Some(generation) = self.cache.focus_generation(post) else {
            return false;
        };
        if !self.cache.reset_stage_retry(post, generation) {
            return false;
        }
        let attempt = self.allocate_attempt();
        let pending = Pending::root(generation, attempt, index, root);
        self.pending.insert(post.clone(), pending);
        true
    }

    fn root_entry<'a>(&'a self, post: &PostId, root: &str) -> Option<(usize, &'a String)> {
        let target = self.targets.iter().find(|target| &target.post == post)?;
        target
            .sources
            .iter()
            .enumerate()
            .find(|(_, known)| known.as_str() == root)
    }
}
