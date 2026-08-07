//! Tier classification for delivery work (plan §3): which urgency
//! class a post's next chunk belongs to, given viewer focus, the
//! inventory picture, and live playback demand. Pure and clock-free.

use crate::focus::FocusState;
use crate::inventory_controller::{InventoryState, Mode};
use crate::PostId;

/// Delivery urgency classes, most urgent first (`T0 < … < T4`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Tier {
    T0PlaybackEmergency,
    T1CurrentTail,
    T2Startability,
    T3Deepening,
    T4Speculative,
}

/// Playback-side urgency inputs for the post under classification.
/// Demand only ever concerns the playing post; callers pass
/// `DemandSignals::default()` for everything else.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DemandSignals {
    /// The gateway saw a player request for bytes not yet on disk.
    pub gateway_demand: bool,
    /// Buffer-ahead of the playhead fell below the emergency threshold.
    pub buffer_below_emergency: bool,
    /// Watch time passed the commitment threshold (plan §3 T1).
    pub viewer_committed: bool,
}

impl DemandSignals {
    fn is_emergency(&self) -> bool {
        self.gateway_demand || self.buffer_below_emergency
    }
}

/// Inventory facts for classifying one post: the control-loop
/// observation flattened, plus this post's own startability status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PostInventory {
    pub mode: Mode,
    pub startable_target_met: bool,
    pub startable_window: usize,
    pub head_complete: bool,
}

impl PostInventory {
    /// Flattens a controller observation for one post's classification.
    pub fn new(state: InventoryState, startable_window: usize, head_complete: bool) -> Self {
        Self {
            mode: state.mode,
            startable_target_met: state.counts.startable >= state.counts.target,
            startable_window,
            head_complete,
        }
    }
}

/// Classifies the post's next chunk of work. `None` means the engine
/// owes this post nothing right now — notably any post outside the
/// focus window: scrolling past a post cancels its work.
pub fn classify(
    post: &PostId,
    focus: &FocusState,
    inventory: PostInventory,
    demand: DemandSignals,
) -> Option<Tier> {
    let distance = focus.distance_of(post)?;
    if distance == 0 && demand.is_emergency() {
        return Some(Tier::T0PlaybackEmergency);
    }
    match inventory.head_complete {
        false => Some(head_tier(distance, inventory)),
        true => tail_tier(distance, inventory.mode, demand),
    }
}

/// Head chunks: the current post is always startability work; other
/// upcoming posts only while the target is unmet. Everything else —
/// beyond-window and scroll-back heads — is speculative.
fn head_tier(distance: i64, inventory: PostInventory) -> Tier {
    let upcoming = (0..inventory.startable_window as i64).contains(&distance);
    match distance == 0 || (upcoming && !inventory.startable_target_met) {
        true => Tier::T2Startability,
        false => Tier::T4Speculative,
    }
}

/// Tail chunks: commitment finishes the current video in any mode;
/// otherwise deepening is comfort-only and scroll-back speculative.
fn tail_tier(distance: i64, mode: Mode, demand: DemandSignals) -> Option<Tier> {
    if distance == 0 && demand.viewer_committed {
        return Some(Tier::T1CurrentTail);
    }
    match (mode, distance >= 0) {
        (Mode::Comfort, true) => Some(Tier::T3Deepening),
        (Mode::Comfort, false) => Some(Tier::T4Speculative),
        (Mode::Hunger, _) => None,
    }
}
