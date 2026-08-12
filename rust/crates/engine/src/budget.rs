//! Data-usage connection budgets derived from one pristine parameter table.

use crate::{DataUsageLevel, EngineParams};

pub fn params_for(level: DataUsageLevel, base: EngineParams) -> EngineParams {
    pin_concurrency(base, base.concurrency(level))
}

fn pin_concurrency(params: EngineParams, concurrency: usize) -> EngineParams {
    EngineParams {
        conservative_concurrency: concurrency,
        balanced_concurrency: concurrency,
        aggressive_concurrency: concurrency,
        ..params
    }
}
