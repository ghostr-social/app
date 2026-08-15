//! The on-disk file family one stored key owns under the store root.

use anyhow::{bail, Result};
use std::path::PathBuf;

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

    /// Every file of the key, partial and completed alike.
    pub fn all(&self, key: &str) -> [PathBuf; 10] {
        [
            self.partial(key),
            self.partial_staging(key),
            self.manifest(key),
            self.manifest(key).with_extension("json.tmp"),
            self.completed(key),
            self.verified(key),
            self.representation(key),
            self.representation(key)
                .with_extension("representation.tmp"),
            self.generation(key),
            self.generation(key).with_extension("json.tmp"),
        ]
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
