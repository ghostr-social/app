use crate::partial_range_disk as disk;
use crate::partial_range_manifest::{IntervalChecksum, RangeManifest};
use anyhow::{ensure, Context as _, Result};
use core::ops::Range;
use std::collections::BTreeSet;
use std::path::Path;
use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _, AsyncWriteExt as _};

const COPY_BLOCK_BYTES: u64 = ghostr_engine::adaptive::REQUEST_SLICE_BYTES;

mod tail;
pub(super) use tail::verified_manifest;

struct StagingWriter<'a> {
    output: &'a mut tokio::fs::File,
    manifest: &'a mut RangeManifest,
}

async fn retained_is_valid(
    source: &Path,
    manifest: &RangeManifest,
    retained: &[Range<u64>],
) -> Result<bool> {
    let mut verified = BTreeSet::new();
    for range in retained {
        for checksum in manifest.checksums_for(range)? {
            let span = checksum.span();
            if !verified.insert((span.start, span.end)) {
                continue;
            }
            if disk::sha256_span(source, &span).await? != checksum.digest() {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

pub(super) async fn manifest_is_valid(source: &Path, manifest: &RangeManifest) -> Result<bool> {
    retained_is_valid(source, manifest, &manifest.ranges()).await
}

pub(super) async fn stage_verified(
    source: &Path,
    staging: &Path,
    source_manifest: &RangeManifest,
    retained: &RangeManifest,
) -> Result<RangeManifest> {
    disk::remove_if_present(staging).await?;
    let mut input = tokio::fs::File::open(source)
        .await
        .context("open policy source video")?;
    let mut output = tokio::fs::File::create(staging)
        .await
        .context("create policy staging video")?;
    let ranges = retained.ranges();
    let mut staged = retained.clone();
    let mut range_index = 0;
    for checksum in source_manifest.checksum_records() {
        let intersections = next_intersections(&checksum.span(), &ranges, &mut range_index);
        let mut writer = StagingWriter {
            output: &mut output,
            manifest: &mut staged,
        };
        copy_verified_record(&mut input, checksum, intersections, &mut writer).await?;
    }
    output
        .sync_all()
        .await
        .context("sync policy staging video")?;
    disk::sync_parent(staging).await?;
    staged.to_json()?;
    Ok(staged)
}

async fn copy_verified_record(
    input: &mut tokio::fs::File,
    checksum: &IntervalChecksum,
    intersections: Vec<Range<u64>>,
    writer: &mut StagingWriter<'_>,
) -> Result<()> {
    if intersections.is_empty() {
        return Ok(());
    }
    let span = checksum.span();
    let bytes = read_record(input, &span).await?;
    ensure!(
        disk::sha256_bytes(&bytes) == checksum.digest(),
        "policy eviction found corrupt retained bytes"
    );
    for intersection in intersections {
        writer.copy_fragments(&span, &bytes, intersection).await?;
    }
    Ok(())
}

async fn read_record(input: &mut tokio::fs::File, span: &Range<u64>) -> Result<Vec<u8>> {
    let length = usize::try_from(span.end.saturating_sub(span.start))
        .context("policy checksum span exceeds address space")?;
    let mut bytes = vec![0; length];
    input.seek(std::io::SeekFrom::Start(span.start)).await?;
    input.read_exact(&mut bytes).await?;
    Ok(bytes)
}

impl StagingWriter<'_> {
    async fn copy_fragments(
        &mut self,
        record: &Range<u64>,
        bytes: &[u8],
        retained: Range<u64>,
    ) -> Result<()> {
        let mut start = retained.start;
        while start < retained.end {
            let end = retained.end.min(start.saturating_add(COPY_BLOCK_BYTES));
            let span = start..end;
            let slice = &bytes[(start - record.start) as usize..(end - record.start) as usize];
            self.output.seek(std::io::SeekFrom::Start(start)).await?;
            self.output.write_all(slice).await?;
            self.manifest
                .record_checksum(span, disk::sha256_bytes(slice))?;
            start = end;
        }
        Ok(())
    }
}

fn next_intersections(
    record: &Range<u64>,
    retained: &[Range<u64>],
    cursor: &mut usize,
) -> Vec<Range<u64>> {
    while retained
        .get(*cursor)
        .is_some_and(|range| range.end <= record.start)
    {
        *cursor += 1;
    }
    let mut index = *cursor;
    let mut intersections = Vec::new();
    while retained
        .get(index)
        .is_some_and(|range| range.start < record.end)
    {
        let range = &retained[index];
        intersections.push(record.start.max(range.start)..record.end.min(range.end));
        if range.end > record.end {
            break;
        }
        index += 1;
    }
    *cursor = index;
    intersections
}
