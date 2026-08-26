//! Storage pressure is not a source failure. When the device has no
//! room left, the bytes that did not land say nothing about the host
//! that sent them: charging the attempt to the source spends its retry
//! budget, and a post whose every mirror is retired stops being served
//! at all. Device pass 3 caught exactly that — sixteen "out of space"
//! refusals inside one second, a `Video unavailable` panel and seven
//! failed player initializations.
//!
//! So a refused write pauses the post until the store can have made
//! room, leaves the source's ledger untouched, and is reported once per
//! refusal decision rather than once per buffer.

use crate::manager::transfers::InternalEvent;
use crate::manager::DeliveryWorker;
use core::time::Duration;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::CapacityRevision;
use ghostr_partial_store::partial_range_store::OutOfSpace;
use log::warn;
use std::sync::Arc;

mod capacity;
pub(crate) use capacity::capacity_changed;

/// What the manager carries between refusals.
pub(crate) struct StorePressure {
    /// Refusal decisions already reported, so a burst of buffers
    /// hitting one decision is logged once.
    reported: u64,
    /// Delay before one external free-space recheck.
    pause: Duration,
    parked: bool,
    wait_generation: u64,
}

impl StorePressure {
    pub(crate) fn new(pause: Duration) -> Self {
        Self {
            reported: 0,
            pause,
            parked: false,
            wait_generation: 0,
        }
    }

    fn claim_report(&mut self, decisions: u64) -> bool {
        if decisions == self.reported {
            return false;
        }
        self.reported = decisions;
        true
    }

    pub(crate) fn report(&mut self, decisions: u64, short: u64) -> Option<u64> {
        self.claim_report(decisions).then_some(short)
    }

    pub(super) fn is_parked(&self) -> bool {
        self.parked
    }

    pub(super) fn retry_delay(&self) -> Duration {
        self.pause
    }

    fn park(&mut self, observed: CapacityRevision) -> Option<CapacityWait> {
        if self.parked {
            return None;
        }
        self.parked = true;
        self.wait_generation = self.wait_generation.saturating_add(1);
        Some(CapacityWait {
            generation: self.wait_generation,
            recheck_after: self.pause,
            observed,
        })
    }

    fn resume(&mut self, generation: u64) {
        if self.wait_generation == generation {
            self.parked = false;
        }
    }

    pub(super) fn clear(&mut self) {
        self.wait_generation = self.wait_generation.saturating_add(1);
        self.reported = 0;
        self.parked = false;
    }

    pub(super) fn focus_changed(&mut self) {
        self.wait_generation = self.wait_generation.saturating_add(1);
        self.parked = false;
    }
}

#[derive(Clone, Copy)]
struct CapacityWait {
    generation: u64,
    recheck_after: Duration,
    observed: CapacityRevision,
}

impl DeliveryWorker {
    /// Replans on every store capacity revision. The policy budgets
    /// origin work to the room the store reports, so a capacity change
    /// must wake planning even when nothing is parked on a refusal.
    pub(super) fn spawn_capacity_replans(&self) {
        let store = Arc::clone(&self.ctx.store);
        let events = self.ctx.events.clone();
        tokio::spawn(async move {
            let mut changes = store.capacity_changes();
            loop {
                tokio::select! {
                    result = changes.changed() => {
                        if result.is_err() {
                            return;
                        }
                        let woke = events.send(InternalEvent::Maintenance(
                            crate::manager::transfers::MaintenanceEvent::StoreCapacityChanged(0),
                        ));
                        if woke.is_err() {
                            return;
                        }
                    }
                    () = events.closed() => return,
                }
            }
        });
    }

    /// Absorbs a chunk that failed on the store rather than on the
    /// network. `true` when it was one, so the caller leaves the
    /// source's retry ledger alone.
    pub(super) fn absorb_store_pressure(&mut self, _post: &PostId, error: &anyhow::Error) -> bool {
        let Some(refusal) = out_of_space(error) else {
            return false;
        };
        self.report_pressure(refusal.short);
        self.park_for_capacity(refusal.capacity_revision());
        true
    }

    pub(super) fn resume_store_capacity(&mut self, generation: u64) {
        self.pressure.resume(generation);
    }

    fn report_pressure(&mut self, short: u64) {
        let decisions = self.ctx.store.refusals();
        let _ = self
            .pressure
            .report(decisions, short)
            .inspect(|short| warn_pressure(*short));
    }

    fn park_for_capacity(&mut self, observed: CapacityRevision) {
        let Some(wait) = self.pressure.park(observed) else {
            return;
        };
        let store = Arc::clone(&self.ctx.store);
        let mut changes = store.capacity_changes();
        let events = self.ctx.events.clone();
        tokio::spawn(async move {
            let changed = tokio::select! {
                changed = capacity_changed(
                    &store,
                    &mut changes,
                    wait.recheck_after,
                    wait.observed,
                ) => changed,
                () = events.closed() => false,
            };
            if changed {
                let _ = events.send(InternalEvent::Maintenance(
                    crate::manager::transfers::MaintenanceEvent::StoreCapacityChanged(
                        wait.generation,
                    ),
                ));
            }
        });
    }
}

fn warn_pressure(short: u64) {
    warn!("Video store has no room for {short} more bytes; pausing the post instead of its source");
}

pub(crate) fn is_store_pressure(error: &anyhow::Error) -> bool {
    out_of_space(error).is_some()
}

/// The shortfall a store refusal carried, wherever it sits in the chain.
fn out_of_space(error: &anyhow::Error) -> Option<&OutOfSpace> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<OutOfSpace>())
}
