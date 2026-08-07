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

use ghostr_engine::PostId;
use crate::delivery_manager::DeliveryWorker;
use ghostr_partial_store::partial_range_store::OutOfSpace;
use log::warn;
use std::time::Duration;

/// What the manager carries between refusals.
pub(crate) struct StorePressure {
    /// Refusal decisions already reported, so a burst of buffers
    /// hitting one decision is logged once.
    reported: u64,
    /// How long a post waits before asking the store again.
    pause: Duration,
}

impl StorePressure {
    pub(crate) fn new(pause: Duration) -> Self {
        Self { reported: 0, pause }
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
}

impl DeliveryWorker {
    /// Absorbs a chunk that failed on the store rather than on the
    /// network. `true` when it was one, so the caller leaves the
    /// source's retry ledger alone.
    pub(crate) fn absorb_store_pressure(&mut self, post: &PostId, error: &anyhow::Error) -> bool {
        let Some(short) = out_of_space(error) else {
            return false;
        };
        self.report_pressure(short);
        self.start_cooldown(post.clone(), self.pressure.pause);
        true
    }

    fn report_pressure(&mut self, short: u64) {
        let decisions = self.ctx.store.refusals();
        let _ = self
            .pressure
            .report(decisions, short)
            .inspect(|short| warn_pressure(*short));
    }
}

fn warn_pressure(short: u64) {
    warn!("Video store has no room for {short} more bytes; pausing the post instead of its source");
}

/// The shortfall a store refusal carried, wherever it sits in the chain.
fn out_of_space(error: &anyhow::Error) -> Option<u64> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<OutOfSpace>())
        .map(|refusal| refusal.short)
}
