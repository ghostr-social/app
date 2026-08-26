use super::*;

impl ReadPlan {
    pub(in super::super) fn capture(
        paths: &StorePaths,
        key: &str,
        entry: &Entry,
        requested: Range<u64>,
    ) -> Result<Option<Self>> {
        Self::capture_with_manifest(paths, key, entry, &entry.manifest, requested)
    }
}
