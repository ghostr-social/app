use crate::partial_range_disk as disk;
use crate::partial_range_manifest::{IntervalChecksum, RangeManifest};
use anyhow::{ensure, Result};
use core::ops::Range;
use std::path::Path;

pub(in crate::partial_range_store) async fn verified_manifest(
    source: &Path,
    source_manifest: &RangeManifest,
    retained: &RangeManifest,
    tail_end: u64,
) -> Result<RangeManifest> {
    let mut verified = retained.clone();
    for checksum in source_manifest.checksum_records() {
        let span = checksum.span();
        if span.start >= tail_end {
            break;
        }
        if span.end <= tail_end {
            verified.record_checksum(span, checksum.digest().to_owned())?;
        } else {
            record_boundary(source, checksum, tail_end, &mut verified).await?;
        }
    }
    verified.to_json()?;
    Ok(verified)
}

async fn record_boundary(
    source: &Path,
    checksum: &IntervalChecksum,
    tail_end: u64,
    manifest: &mut RangeManifest,
) -> Result<()> {
    let record = checksum.span();
    let bytes = disk::read_span(source, &record).await?;
    ensure!(
        disk::sha256_bytes(&bytes) == checksum.digest(),
        "policy eviction found corrupt retained bytes"
    );
    let retained = record.start..tail_end.min(record.end);
    record_fragments(manifest, &record, &bytes, retained)
}

fn record_fragments(
    manifest: &mut RangeManifest,
    record: &Range<u64>,
    bytes: &[u8],
    retained: Range<u64>,
) -> Result<()> {
    let mut start = retained.start;
    while start < retained.end {
        let end = retained
            .end
            .min(start.saturating_add(ghostr_engine::adaptive::REQUEST_SLICE_BYTES));
        let slice = &bytes[(start - record.start) as usize..(end - record.start) as usize];
        manifest.record_checksum(start..end, disk::sha256_bytes(slice))?;
        start = end;
    }
    Ok(())
}
