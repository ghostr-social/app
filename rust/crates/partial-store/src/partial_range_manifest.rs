use anyhow::{bail, Result};
use std::ops::Range;

mod checksums;
mod format;

#[cfg(test)]
mod tests;

/// Normalized coverage plus a local checksum for every committed interval.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RangeManifest {
    total_len: Option<u64>,
    ranges: Vec<(u64, u64)>,
    checksums: Vec<IntervalChecksum>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IntervalChecksum {
    pub(super) start: u64,
    pub(super) end: u64,
    pub(super) sha256: String,
}

impl IntervalChecksum {
    pub(crate) fn span(&self) -> Range<u64> {
        self.start..self.end
    }

    pub(crate) fn digest(&self) -> &str {
        &self.sha256
    }
}

impl RangeManifest {
    pub(crate) fn total_len(&self) -> Option<u64> {
        self.total_len
    }

    pub fn set_total_len(&mut self, len: u64) -> Result<()> {
        if self.total_len.is_some_and(|current| current != len) {
            bail!("total length is already declared with a different value");
        }
        if self.ranges.last().is_some_and(|&(_, end)| end > len) {
            bail!("total length is shorter than the stored bytes");
        }
        self.total_len = Some(len);
        Ok(())
    }

    pub fn insert(&mut self, span: Range<u64>) -> Result<()> {
        if span.start >= span.end {
            return Ok(());
        }
        if self.total_len.is_some_and(|len| span.end > len) {
            bail!("range extends past the declared total length");
        }
        self.ranges.push((span.start, span.end));
        self.ranges.sort_unstable();
        self.ranges = coalesce(&self.ranges);
        Ok(())
    }

    pub(crate) fn remove(&mut self, span: &Range<u64>) -> u64 {
        if span.start >= span.end {
            return 0;
        }
        let before = self.covered_bytes();
        let mut remaining = Vec::new();
        for &(start, end) in &self.ranges {
            retain_outside(&mut remaining, start..end, span);
        }
        self.ranges = remaining;
        self.checksums.clear();
        before.saturating_sub(self.covered_bytes())
    }

    pub fn ranges(&self) -> Vec<Range<u64>> {
        self.ranges.iter().map(|&(start, end)| start..end).collect()
    }

    pub(crate) fn covered_bytes(&self) -> u64 {
        self.ranges.iter().map(|(start, end)| end - start).sum()
    }

    pub(crate) fn contains(&self, span: &Range<u64>) -> bool {
        span.start >= span.end
            || self
                .ranges
                .iter()
                .any(|&(start, end)| start <= span.start && span.end <= end)
    }

    pub(crate) fn missing_within(&self, span: &Range<u64>) -> Vec<Range<u64>> {
        let mut missing = Vec::new();
        let mut cursor = span.start;
        for &(start, end) in &self.ranges {
            if start > cursor && cursor < span.end {
                missing.push(cursor..span.end.min(start));
            }
            cursor = cursor.max(end);
        }
        if cursor < span.end {
            missing.push(cursor..span.end);
        }
        missing
    }

    pub fn is_complete(&self) -> bool {
        match self.total_len {
            Some(0) => self.ranges.is_empty(),
            Some(len) => self.ranges == [(0, len)],
            None => false,
        }
    }

    pub(crate) fn to_json(&self) -> Result<String> {
        format::encode(self.total_len, &self.ranges, &self.checksums)
    }

    pub(crate) fn from_json(text: &str) -> Result<Self> {
        let disk = format::decode(text)?;
        Ok(Self {
            total_len: disk.total_len,
            ranges: format::checksum_ranges(&disk.intervals),
            checksums: disk.intervals,
        })
    }
}

fn retain_outside(remaining: &mut Vec<(u64, u64)>, stored: Range<u64>, removed: &Range<u64>) {
    if removed.end <= stored.start || removed.start >= stored.end {
        remaining.push((stored.start, stored.end));
    } else {
        if stored.start < removed.start {
            remaining.push((stored.start, removed.start.min(stored.end)));
        }
        if removed.end < stored.end {
            remaining.push((removed.end.max(stored.start), stored.end));
        }
    }
}

fn coalesce(sorted: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for &(start, end) in sorted {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    merged
}
