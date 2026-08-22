use super::{coalesce, IntervalChecksum};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

const MANIFEST_VERSION: u8 = 2;
// WARP Table 6's upper bound for an adaptive cancellation block.
const MAX_INTERVAL_BYTES: u64 = 512 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DiskManifest {
    version: u8,
    pub(super) total_len: Option<u64>,
    pub(super) intervals: Vec<IntervalChecksum>,
}

pub(super) fn encode(
    total_len: Option<u64>,
    ranges: &[(u64, u64)],
    intervals: &[IntervalChecksum],
) -> Result<String> {
    validate_intervals(intervals, total_len)?;
    ensure!(
        checksum_ranges(intervals) == ranges,
        "manifest coverage is not fully checksummed"
    );
    serde_json::to_string(&DiskManifest {
        version: MANIFEST_VERSION,
        total_len,
        intervals: intervals.to_vec(),
    })
    .context("encode partial range manifest")
}

pub(super) fn decode(text: &str) -> Result<DiskManifest> {
    let disk: DiskManifest = serde_json::from_str(text).context("parse range manifest")?;
    ensure!(
        disk.version == MANIFEST_VERSION,
        "unsupported range manifest"
    );
    validate_intervals(&disk.intervals, disk.total_len)?;
    Ok(disk)
}

fn validate_intervals(intervals: &[IntervalChecksum], total: Option<u64>) -> Result<()> {
    let mut previous_end = 0;
    for (index, interval) in intervals.iter().enumerate() {
        ensure!(interval.start < interval.end, "empty checksum interval");
        ensure!(
            interval.end - interval.start <= MAX_INTERVAL_BYTES,
            "checksum interval exceeds WARP cancellation block"
        );
        ensure!(
            index == 0 || interval.start >= previous_end,
            "overlapping intervals"
        );
        ensure!(
            total.is_none_or(|len| interval.end <= len),
            "interval exceeds total"
        );
        ensure!(valid_digest(&interval.sha256), "invalid interval checksum");
        previous_end = interval.end;
    }
    Ok(())
}

pub(super) fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(super) fn checksum_ranges(checksums: &[IntervalChecksum]) -> Vec<(u64, u64)> {
    let pairs: Vec<_> = checksums
        .iter()
        .map(|item| (item.start, item.end))
        .collect();
    coalesce(&pairs)
}
