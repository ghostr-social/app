//! Causal readiness ETA for a progressive post.
//!
//! A plan predicts when a post becomes startable only if it was computed
//! for the focus generation the watcher currently tracks; an older plan
//! describes a window the user has already left (paper §9.8: wait only
//! inside a small grace interval predicted from delivery evidence).

use crate::api::delivery::snapshots::DeliverySnapshot;
use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};
use std::sync::Arc;
use tokio::sync::Notify;

/// Plans are republished on every material event and their delivery
/// estimates drift by a few milliseconds each time; an event per drift
/// would flood the feed with progress updates that change no decision.
/// The ETA is therefore rounded up to this bucket, which is well under
/// the selector's grace interval.
const ETA_BUCKET_MS: u64 = 100;

/// Wakes the watcher when a new plan is published; idle without a handle.
pub(crate) fn plan_notifier(delivery: Option<&DeliveryHandle>) -> Arc<Notify> {
    delivery.map_or_else(|| Arc::new(Notify::new()), DeliveryHandle::plan_notifier)
}

/// Attaches the causal ETA to a snapshot that is not yet startable.
pub(crate) fn with_causal_eta(
    mut snapshot: DeliverySnapshot,
    delivery: Option<&DeliveryHandle>,
    focus_generation: Option<u64>,
    post: &str,
) -> DeliverySnapshot {
    if !snapshot.startable && !snapshot.failed {
        snapshot.eta_ms = causal_eta_ms(delivery, focus_generation, post);
    }
    snapshot
}

/// The smallest expected delivery time among the latest causal plan's
/// allocations that add playable media for `post`; `None` when no causal
/// plan or no such allocation exists.
fn causal_eta_ms(
    delivery: Option<&DeliveryHandle>,
    focus_generation: Option<u64>,
    post: &str,
) -> Option<u64> {
    let plan = delivery?.latest_plan()?;
    if !is_causal(&plan, focus_generation) {
        return None;
    }
    plan.plan
        .allocations
        .iter()
        .filter(|allocation| allocation.post.as_str() == post)
        .filter(|allocation| allocation.expected_playable_gain_ms > 0)
        .map(|allocation| allocation.utility.expected_delivery_ms)
        .min()
        .map(bucketed)
}

const fn bucketed(eta_ms: u64) -> u64 {
    eta_ms.div_ceil(ETA_BUCKET_MS).saturating_mul(ETA_BUCKET_MS)
}

fn is_causal(plan: &PlanEvidence, focus_generation: Option<u64>) -> bool {
    focus_generation.is_some() && plan.focus_generation == focus_generation
}
