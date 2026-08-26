use super::EvictionOutcome;
use crate::partial_range_disk::Entry;
use crate::partial_range_manifest::RangeManifest;
use anyhow::{ensure, Context as _, Result};
use core::ops::Range;

pub(super) struct EvictionPlan {
    pub(super) source: RangeManifest,
    pub(super) retained: RangeManifest,
    pub(super) accounted: u64,
    pub(super) completed: bool,
    pub(super) outcome: EvictionOutcome,
}

impl EvictionPlan {
    pub(super) fn tail_end(&self) -> Option<u64> {
        let end = self.retained.ranges().last()?.end;
        let mut prefix = self.source.clone();
        prefix.remove(&(end..u64::MAX));
        (prefix.ranges() == self.retained.ranges()).then_some(end)
    }
}

pub(super) fn build(entry: &Entry, ranges: &[Range<u64>]) -> EvictionPlan {
    let source = entry.manifest.clone();
    let mut retained = source.clone();
    let requested = ranges
        .iter()
        .map(|range| retained.remove(range))
        .sum::<u64>();
    if entry.completion.is_some() && requested > 0 {
        retained = RangeManifest::default();
    }
    let outcome = EvictionOutcome::between(&source, &retained);
    EvictionPlan {
        source,
        retained,
        accounted: entry.accounted,
        completed: entry.completion.is_some(),
        outcome,
    }
}

pub(super) fn ensure_current(entry: Option<&Entry>, plan: &EvictionPlan) -> Result<()> {
    let entry = entry.context("policy source entry disappeared")?;
    ensure!(
        entry.completion.is_some() == plan.completed,
        "policy source changed"
    );
    ensure!(
        entry.accounted == plan.accounted,
        "policy accounting changed"
    );
    ensure!(
        entry.manifest == plan.source,
        "policy source ranges changed"
    );
    Ok(())
}
