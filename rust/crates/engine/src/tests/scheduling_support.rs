use crate::catalog::Catalog;
use crate::focus::{FocusState, FocusUpdate};
use crate::inventory_controller::{
    InventoryController, InventoryState, Mode, PresentRanges,
};
use crate::scoring::{next_work, ChunkRequest, NextWorkContext};
use crate::tests::support::ids;
use crate::tiers::{DemandSignals, PostInventory};
use crate::{ByteRange, EngineParams, PostId};
use std::collections::HashMap;

pub fn focus_at(window: &[&str], current_index: usize, watch_ms: u64) -> FocusState {
    let mut focus = FocusState::new();
    focus.update_focus(FocusUpdate {
        window: ids(window),
        current_index,
        watch_ms,
    });
    focus
}

pub fn state(mode: Mode, target_met: bool, head_complete: bool) -> PostInventory {
    PostInventory {
        mode,
        startable_target_met: target_met,
        startable_window: 6,
        head_complete,
    }
}

pub fn hunger(head_complete: bool) -> PostInventory {
    state(Mode::Hunger, false, head_complete)
}

pub fn comfort(head_complete: bool) -> PostInventory {
    state(Mode::Comfort, true, head_complete)
}

/// Owns everything `next_work` borrows so tests can build scenarios
/// declaratively; the mode comes from a real controller observation.
pub struct WorkBench {
    pub catalog: Catalog,
    pub focus: FocusState,
    pub params: EngineParams,
    pub demand: DemandSignals,
    pub present: HashMap<PostId, Vec<ByteRange>>,
}

impl WorkBench {
    pub fn new() -> Self {
        Self {
            catalog: Catalog::new(),
            focus: FocusState::new(),
            params: EngineParams::default(),
            demand: DemandSignals::default(),
            present: HashMap::new(),
        }
    }

    pub fn run(&self) -> Vec<ChunkRequest> {
        let inventory = self.observe();
        let present = |post: &PostId| self.present.get(post).cloned().unwrap_or_default();
        let host_factor = |_: &PostId| 1.0;
        next_work(&NextWorkContext {
            catalog: &self.catalog,
            focus: &self.focus,
            params: &self.params,
            inventory,
            demand: self.demand,
            present: &present,
            host_factor: &host_factor,
        })
    }

    fn observe(&self) -> InventoryState {
        let mut ranges = PresentRanges::new();
        for (post, have) in &self.present {
            ranges.set(post.clone(), have.clone());
        }
        let mut controller = InventoryController::new(self.params);
        controller.inventory_state(&self.catalog, &self.focus, &ranges)
    }
}
