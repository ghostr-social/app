use crate::catalog::Catalog;
use crate::focus::FocusState;
use crate::inventory_controller::{InventoryController, InventoryState, PresentRanges};
use crate::media_timeline::PlaybackWindow;
use crate::scoring::{next_work, ChunkRequest, NextWorkContext};
use crate::tiers::DemandSignals;
use crate::{ByteRange, EngineParams, PostId};
use std::collections::HashMap;

/// Owns everything `next_work` borrows so tests can build scenarios
/// declaratively; the mode comes from a real controller observation.
pub(in crate::tests) struct WorkBench {
    pub(in crate::tests) catalog: Catalog,
    pub(in crate::tests) focus: FocusState,
    pub(in crate::tests) params: EngineParams,
    pub(in crate::tests) demand: DemandSignals,
    pub(in crate::tests) present: HashMap<PostId, Vec<ByteRange>>,
    pub(in crate::tests) head_seconds: HashMap<PostId, u64>,
    pub(in crate::tests) tail_end: HashMap<PostId, u64>,
    pub(in crate::tests) media_window: HashMap<PostId, PlaybackWindow>,
    pub(in crate::tests) direct_range: HashMap<PostId, ByteRange>,
}

impl WorkBench {
    pub(in crate::tests) fn new() -> Self {
        Self {
            catalog: Catalog::new(),
            focus: FocusState::new(),
            params: EngineParams::default(),
            demand: DemandSignals::default(),
            present: HashMap::new(),
            head_seconds: HashMap::new(),
            tail_end: HashMap::new(),
            media_window: HashMap::new(),
            direct_range: HashMap::new(),
        }
    }

    pub(in crate::tests) fn run(&self) -> Vec<ChunkRequest> {
        let inventory = self.observe();
        let present = |post: &PostId| self.present.get(post).cloned().unwrap_or_default();
        let head_seconds = |post: &PostId| {
            self.head_seconds
                .get(post)
                .copied()
                .unwrap_or(self.params.head_seconds)
        };
        next_work(&NextWorkContext {
            catalog: &self.catalog,
            focus: &self.focus,
            params: &self.params,
            inventory,
            demand: self.demand,
            present: &present,
            host_factor: &|_| 1.0,
            head_seconds: &head_seconds,
            tail_end: &|post| self.tail_end.get(post).copied(),
            media_window: &|post| self.media_window.get(post).copied(),
            direct_range: &|post| self.direct_range.get(post).copied(),
        })
    }

    pub(in crate::tests) fn observe(&self) -> InventoryState {
        let mut ranges = PresentRanges::new();
        for (post, have) in &self.present {
            ranges.set(post.clone(), have.clone());
        }
        let target = |post: &PostId| {
            self.head_seconds
                .get(post)
                .copied()
                .unwrap_or(self.params.head_seconds)
        };
        InventoryController::new(self.params).inventory_state_with(
            &self.catalog,
            &self.focus,
            &ranges,
            &target,
        )
    }
}
