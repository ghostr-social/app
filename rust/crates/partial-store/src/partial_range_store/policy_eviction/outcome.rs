use crate::partial_range_manifest::RangeManifest;
use core::ops::Range;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvictionOutcome {
    freed_bytes: u64,
    ranges: Vec<Range<u64>>,
}

impl EvictionOutcome {
    pub fn freed_bytes(&self) -> u64 {
        self.freed_bytes
    }

    pub fn ranges(&self) -> &[Range<u64>] {
        &self.ranges
    }

    pub(super) fn between(source: &RangeManifest, retained: &RangeManifest) -> Self {
        let ranges: Vec<_> = source
            .ranges()
            .into_iter()
            .flat_map(|span| retained.missing_within(&span))
            .collect();
        let freed_bytes = ranges.iter().map(|range| range.end - range.start).sum();
        Self {
            freed_bytes,
            ranges,
        }
    }
}
