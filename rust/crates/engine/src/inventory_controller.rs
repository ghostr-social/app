//! Inventory control loop (plan §3): counts how many upcoming posts
//! are startable and switches between hunger and comfort with
//! hysteresis. Pure bookkeeping — no IO, no clocks.

use crate::catalog::Catalog;
use crate::focus::FocusState;
use crate::{ByteRange, EngineParams, PostId};
use std::collections::HashMap;

mod startability;
mod timeline;
pub use startability::is_startable;
use startability::{count_inventory, InventoryInputs};

/// Control-loop mode: hunger races for startability, comfort deepens.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mode {
    Hunger,
    Comfort,
}

/// Byte ranges already on disk, per post. Posts never set are empty.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PresentRanges {
    by_post: HashMap<PostId, Vec<ByteRange>>,
}

impl PresentRanges {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, post: PostId, ranges: Vec<ByteRange>) {
        self.by_post.insert(post, ranges);
    }

    fn ranges(&self, post: &PostId) -> &[ByteRange] {
        self.by_post.get(post).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Startability inventory over the upcoming window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventoryCounts {
    /// Posts actually considered: current + ahead, capped at
    /// `startable_window`.
    pub(crate) considered: usize,
    /// How many consecutive posts from current are startable now.
    pub(crate) startable: usize,
    /// Effective target: the configured target, never above
    /// `considered`.
    pub(crate) target: usize,
}

/// One control-loop observation: the counts and the resulting mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InventoryState {
    pub(crate) counts: InventoryCounts,
    pub mode: Mode,
}

impl InventoryState {
    pub fn current_startable(self) -> bool {
        self.counts.startable > 0
    }
}

/// Remembers the mode across observations so hysteresis has memory.
/// A fresh controller starts hungry: nothing is fetched yet.
#[derive(Clone, Debug)]
pub struct InventoryController {
    params: EngineParams,
    mode: Mode,
}

impl InventoryController {
    pub fn new(params: EngineParams) -> Self {
        Self {
            params,
            mode: Mode::Hunger,
        }
    }

    #[cfg(test)]
    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    /// Recounts startable posts in the upcoming window and applies the
    /// hysteresis rule. The returned mode is remembered for the next
    /// observation.
    pub fn inventory_state(
        &mut self,
        catalog: &Catalog,
        focus: &FocusState,
        present: &PresentRanges,
    ) -> InventoryState {
        let seconds = self.params.head_seconds;
        self.inventory_state_with(catalog, focus, present, &|_| seconds)
    }

    pub fn inventory_state_with(
        &mut self,
        catalog: &Catalog,
        focus: &FocusState,
        present: &PresentRanges,
        head_seconds: &dyn Fn(&PostId) -> u64,
    ) -> InventoryState {
        let counts = count_inventory(InventoryInputs {
            catalog,
            focus,
            present,
            params: &self.params,
            head_seconds,
        });
        self.mode = next_mode(self.mode, counts.startable, counts.target);
        InventoryState {
            counts,
            mode: self.mode,
        }
    }
}

/// Hysteresis rule (plan §3): enter comfort once the target is met;
/// fall back to hunger only below `target - 1`, so losing a single
/// startable post cannot flap the mode.
pub(crate) fn next_mode(current: Mode, startable: usize, target: usize) -> Mode {
    if startable >= target {
        Mode::Comfort
    } else if startable + 1 < target {
        Mode::Hunger
    } else {
        current
    }
}
