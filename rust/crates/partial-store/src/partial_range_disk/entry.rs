use super::{file_len, load_manifest};
use crate::partial_range_completion::{self as completion, Completion};
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_paths::StorePaths;
use anyhow::{ensure, Result};

/// In-memory bookkeeping for one stored key, rebuilt lazily from disk.
/// `completion` is `None` while the key is still partial.
pub struct Entry {
    pub manifest: RangeManifest,
    pub accounted: u64,
    pub completion: Option<Completion>,
    /// Monotonic use counter driving eviction order; `0` until the key
    /// is read or written in this run, which makes untouched keys the
    /// first candidates.
    pub touched: u64,
}

impl Entry {
    fn partial(manifest: RangeManifest) -> Self {
        let accounted = manifest.covered_bytes();
        Self {
            manifest,
            accounted,
            completion: None,
            touched: 0,
        }
    }

    fn completed(len: u64, completion: Completion, manifest: RangeManifest) -> Result<Self> {
        ensure!(
            manifest.total_len() == Some(len),
            "completed length mismatch"
        );
        ensure!(manifest.is_complete(), "completed manifest is incomplete");
        Ok(Self {
            manifest,
            accounted: len,
            completion: Some(completion),
            touched: 0,
        })
    }
}

pub async fn load_entry(paths: &StorePaths, key: &str) -> Result<Entry> {
    if let Some(len) = file_len(&paths.completed(key)).await? {
        let completion = completion::recorded(&paths.verified(key)).await?;
        let manifest = load_manifest(&paths.manifest(key)).await?;
        return Entry::completed(len, completion, manifest);
    }
    Ok(Entry::partial(load_manifest(&paths.manifest(key)).await?))
}
