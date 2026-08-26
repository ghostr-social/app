use super::{format, IntervalChecksum, RangeManifest};
use anyhow::{ensure, Result};
use core::ops::Range;

impl RangeManifest {
    pub(crate) fn checksum_span_for_write(&self, span: Range<u64>) -> Range<u64> {
        self.checksums.iter().fold(span, |merged, checksum| {
            let known = checksum.span();
            if known.start < merged.end && merged.start < known.end {
                merged.start.min(known.start)..merged.end.max(known.end)
            } else {
                merged
            }
        })
    }

    pub(crate) fn record_checksum(&mut self, span: Range<u64>, sha256: String) -> Result<()> {
        ensure!(self.contains(&span), "checksum interval is not present");
        ensure!(
            format::valid_digest(&sha256),
            "checksum is not lowercase SHA-256"
        );
        self.checksums
            .retain(|known| known.end <= span.start || known.start >= span.end);
        self.checksums.push(IntervalChecksum {
            start: span.start,
            end: span.end,
            sha256,
        });
        self.checksums.sort_by_key(|known| known.start);
        Ok(())
    }

    pub(crate) fn checksums_for(&self, span: &Range<u64>) -> Result<Vec<IntervalChecksum>> {
        let checksums: Vec<_> = self
            .checksums
            .iter()
            .filter(|known| known.start < span.end && span.start < known.end)
            .cloned()
            .collect();
        ensure!(covers(&checksums, span), "stored range is not checksummed");
        Ok(checksums)
    }

    pub(crate) fn checksum_records(&self) -> &[IntervalChecksum] {
        &self.checksums
    }
}

fn covers(checksums: &[IntervalChecksum], span: &Range<u64>) -> bool {
    let mut cursor = span.start;
    for checksum in checksums {
        if checksum.start > cursor {
            return false;
        }
        cursor = cursor.max(checksum.end);
    }
    cursor >= span.end
}
