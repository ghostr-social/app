//! The on-disk file family one stored key owns under the store root.

use anyhow::{bail, Result};
use std::path::PathBuf;

mod transform;
pub(crate) use transform::TransformPaths;

/// Names every file a key can own. Keeping them in one place is what
/// makes eviction total: `all` is the removal list, so a new file kind
/// cannot be forgotten there.
pub struct StorePaths {
    root: PathBuf,
}

impl StorePaths {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// The finished file, served once the key left the partial pool.
    pub fn completed(&self, key: &str) -> PathBuf {
        self.named(key, "video")
    }

    /// The sparse file the downloader writes into.
    pub fn partial(&self, key: &str) -> PathBuf {
        self.named(key, "part")
    }

    pub fn partial_staging(&self, key: &str) -> PathBuf {
        self.named(key, "part.tmp")
    }

    pub fn policy_staging(&self, key: &str) -> PathBuf {
        self.named(key, "part.evict")
    }

    pub fn policy_manifest_staging(&self, key: &str) -> PathBuf {
        self.named(key, "ranges.evict")
    }

    pub fn policy_manifest_staging_temp(&self, key: &str) -> PathBuf {
        self.named(key, "ranges.evict.tmp")
    }

    pub fn policy_data_backup(&self, key: &str) -> PathBuf {
        self.named(key, "part.evict.old")
    }

    pub fn policy_manifest_backup(&self, key: &str) -> PathBuf {
        self.named(key, "ranges.evict.old")
    }

    pub fn policy_intent(&self, key: &str) -> PathBuf {
        self.named(key, "evict.intent")
    }

    pub fn policy_intent_staging(&self, key: &str) -> PathBuf {
        self.named(key, "evict.intent.tmp")
    }

    pub fn sparse_intent(&self, key: &str) -> PathBuf {
        self.named(key, "sparse.intent")
    }

    pub fn sparse_intent_staging(&self, key: &str) -> PathBuf {
        self.named(key, "sparse.intent.tmp")
    }

    pub fn single_response(&self, key: &str) -> PathBuf {
        self.named(key, "response.part")
    }

    pub fn single_response_manifest(&self, key: &str) -> PathBuf {
        self.named(key, "response.ranges")
    }

    pub fn manifest_backup(&self, key: &str) -> PathBuf {
        self.named(key, "ranges.prev")
    }

    pub fn generation_backup(&self, key: &str) -> PathBuf {
        self.named(key, "generation.prev")
    }

    pub fn single_response_commit(&self, key: &str) -> PathBuf {
        self.named(key, "response.commit")
    }

    pub fn single_response_artifacts(&self, key: &str) -> [PathBuf; 7] {
        [
            self.single_response(key),
            self.single_response_manifest(key),
            self.single_response_manifest(key)
                .with_extension("json.tmp"),
            self.partial_staging(key),
            self.manifest_backup(key),
            self.generation_backup(key),
            self.single_response_commit(key),
        ]
    }

    /// The persisted set of present ranges of the partial file.
    pub fn manifest(&self, key: &str) -> PathBuf {
        self.named(key, "ranges.json")
    }

    /// Marker written only for bytes that matched an advertised digest.
    pub fn verified(&self, key: &str) -> PathBuf {
        self.named(key, "verified")
    }

    pub fn representation(&self, key: &str) -> PathBuf {
        self.named(key, "representation")
    }

    pub fn generation(&self, key: &str) -> PathBuf {
        self.named(key, "generation.json")
    }

    /// Every payload file of the key. Policy intent is deliberately
    /// excluded: destructive cleanup removes that authority last.
    pub fn all(&self, key: &str) -> Vec<PathBuf> {
        let mut paths = vec![
            self.partial(key),
            self.partial_staging(key),
            self.policy_staging(key),
            self.policy_manifest_staging(key),
            self.policy_manifest_staging_temp(key),
            self.policy_data_backup(key),
            self.policy_manifest_backup(key),
            self.policy_intent_staging(key),
            self.sparse_intent(key),
            self.sparse_intent_staging(key),
            self.single_response(key),
            self.single_response_manifest(key),
            self.single_response_manifest(key)
                .with_extension("json.tmp"),
            self.manifest(key),
            self.manifest(key).with_extension("json.tmp"),
            self.manifest_backup(key),
            self.completed(key),
            self.verified(key),
            self.representation(key),
            self.representation(key)
                .with_extension("representation.tmp"),
            self.generation(key),
            self.generation(key).with_extension("json.tmp"),
            self.generation_backup(key),
            self.single_response_commit(key),
        ];
        paths.extend(self.transform(key).all());
        paths
    }

    pub(crate) fn transform<'a>(&'a self, key: &'a str) -> TransformPaths<'a> {
        TransformPaths::new(self, key)
    }

    fn named(&self, key: &str, extension: &str) -> PathBuf {
        self.root.join(format!("{key}.{extension}"))
    }
}

/// Keys become file names, so they may not carry separators or dots.
pub fn validate_key(key: &str) -> Result<()> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_');
    if !key.is_empty() && key.chars().all(allowed) {
        return Ok(());
    }
    bail!("partial store keys must be alphanumeric with dashes or underscores")
}
