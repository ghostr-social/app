//! Data-usage budgets (plan §3): one base parameter table scaled per
//! `DataUsageLevel`. Pure — always derive from the pristine base on a
//! level change, never from an already-scaled result.

use crate::{DataUsageLevel, EngineParams};

/// Scales the base parameters for one data-usage level. Concurrency
/// is pinned to the level's slot (2/3/4 by default), so the result
/// answers `concurrency(..)` identically for any level asked.
pub fn params_for(level: DataUsageLevel, base: EngineParams) -> EngineParams {
    let scaled = match level {
        DataUsageLevel::Conservative => conservative(base),
        DataUsageLevel::Balanced => base,
        DataUsageLevel::Aggressive => aggressive(base),
    };
    pin_concurrency(scaled, base.concurrency(level))
}

/// Conservative halves the head budget and narrows the lookahead;
/// target and window never drop below one (and window never below
/// target).
fn conservative(base: EngineParams) -> EngineParams {
    let target = base.startable_target.saturating_sub(1).max(1);
    EngineParams {
        head_seconds: base.head_seconds.div_ceil(2),
        head_cap_bytes: (base.head_cap_bytes / 2).max(1),
        startable_target: target,
        startable_window: base.startable_window.saturating_sub(2).max(target),
        ..base
    }
}

/// Aggressive grows the head budget by half and widens the lookahead.
fn aggressive(base: EngineParams) -> EngineParams {
    EngineParams {
        head_seconds: base.head_seconds.saturating_mul(3) / 2,
        head_cap_bytes: base.head_cap_bytes.saturating_mul(2),
        startable_target: base.startable_target.saturating_add(1),
        startable_window: base.startable_window.saturating_add(2),
        ..base
    }
}

fn pin_concurrency(params: EngineParams, concurrency: usize) -> EngineParams {
    EngineParams {
        conservative_concurrency: concurrency,
        balanced_concurrency: concurrency,
        aggressive_concurrency: concurrency,
        ..params
    }
}
