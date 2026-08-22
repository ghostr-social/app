use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use anyhow::Result;
use std::path::Path;

pub(super) async fn complete(path: &Path, total: u64) -> Result<RangeManifest> {
    let mut manifest = RangeManifest::default();
    manifest.set_total_len(total)?;
    manifest.insert(0..total)?;
    let whole = 0..total;
    for (span, checksum) in disk::checksum_blocks(path, std::slice::from_ref(&whole)).await? {
        manifest.record_checksum(span, checksum)?;
    }
    Ok(manifest)
}
