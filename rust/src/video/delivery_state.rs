//! The delivery manager's owned engine state: catalog, focus, budget,
//! and the inventory control loop. Pure bookkeeping — no IO.

use crate::engine::budget::params_for;
use crate::engine::catalog::Catalog;
use crate::engine::focus::{FocusState, FocusUpdate};
use crate::engine::inventory_controller::{InventoryController, InventoryState, PresentRanges};
use crate::engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId};
use crate::video::delivery_events::DeliveryFocus;
use std::collections::HashSet;

pub struct DeliveryState {
    catalog: Catalog,
    focus: FocusState,
    controller: InventoryController,
    base: EngineParams,
    effective: EngineParams,
    level: DataUsageLevel,
}

impl DeliveryState {
    pub fn new(base: EngineParams, level: DataUsageLevel) -> Self {
        let effective = params_for(level, base);
        Self {
            catalog: Catalog::new(),
            focus: FocusState::new(),
            controller: InventoryController::new(effective),
            base,
            effective,
            level,
        }
    }

    /// Applies a focus replacement. Progressive posts land in the
    /// catalog; the returned ids (which double as gateway and store
    /// keys) refresh the servable-post registry.
    pub fn apply_focus(&mut self, update: DeliveryFocus) -> Vec<String> {
        let mut window = Vec::new();
        let mut progressive = Vec::new();
        for item in update.items {
            window.push(item.post.clone());
            if item.meta.delivery == DeliveryKind::Progressive {
                progressive.push(item.post.as_str().to_owned());
                self.catalog.upsert(item.post, item.meta);
            }
        }
        self.focus.update_focus(FocusUpdate {
            window,
            current_index: update.current_index,
            watch_ms: update.watch_ms,
        });
        progressive
    }

    /// Re-derives the budgeted parameters from the pristine base
    /// (never from an already-scaled result).
    pub fn apply_level(&mut self, level: DataUsageLevel) {
        self.level = level;
        self.effective = params_for(level, self.base);
        self.controller = InventoryController::new(self.effective);
    }

    pub fn observe_inventory(&mut self, present: &PresentRanges) -> InventoryState {
        self.controller
            .inventory_state(&self.catalog, &self.focus, present)
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    pub fn focus(&self) -> &FocusState {
        &self.focus
    }

    pub fn params(&self) -> &EngineParams {
        &self.effective
    }

    pub fn concurrency(&self) -> usize {
        self.effective.concurrency(self.level)
    }

    /// Window posts in scroll order, deduplicated.
    pub fn window_posts(&self) -> Vec<PostId> {
        let mut seen = HashSet::new();
        self.focus
            .window()
            .iter()
            .filter(|post| seen.insert((*post).clone()))
            .cloned()
            .collect()
    }
}
