use super::StorePaths;
use std::path::PathBuf;

pub(crate) struct TransformPaths<'a> {
    paths: &'a StorePaths,
    key: &'a str,
}

impl<'a> TransformPaths<'a> {
    pub(crate) const fn new(paths: &'a StorePaths, key: &'a str) -> Self {
        Self { paths, key }
    }

    pub(crate) fn data(&self) -> PathBuf {
        self.paths.named(self.key, "transform.video")
    }

    pub(crate) fn manifest(&self) -> PathBuf {
        self.paths.named(self.key, "transform.ranges")
    }

    pub(crate) fn identity(&self) -> PathBuf {
        self.paths.named(self.key, "transform.representation")
    }

    pub(crate) fn record(&self) -> PathBuf {
        self.paths.named(self.key, "transform.json")
    }

    pub(crate) fn record_staging(&self) -> PathBuf {
        self.paths.named(self.key, "transform.record")
    }

    pub(crate) fn data_backup(&self) -> PathBuf {
        self.paths.named(self.key, "video.transform-prev")
    }

    pub(crate) fn manifest_backup(&self) -> PathBuf {
        self.paths.named(self.key, "ranges.transform-prev")
    }

    pub(crate) fn identity_backup(&self) -> PathBuf {
        self.paths.named(self.key, "representation.transform-prev")
    }

    pub(crate) fn verified_backup(&self) -> PathBuf {
        self.paths.named(self.key, "verified.transform-prev")
    }

    pub(crate) fn commit(&self) -> PathBuf {
        self.paths.named(self.key, "transform.commit")
    }

    pub(crate) fn all(&self) -> Vec<PathBuf> {
        let manifest = self.manifest();
        let identity = self.identity();
        let record = self.record();
        let record_staging = self.record_staging();
        vec![
            self.data(),
            manifest.clone(),
            manifest.with_extension("json.tmp"),
            identity.clone(),
            identity.with_extension("representation.tmp"),
            record.clone(),
            record.with_extension("json.tmp"),
            record_staging.clone(),
            record_staging.with_extension("json.tmp"),
            self.data_backup(),
            self.manifest_backup(),
            self.identity_backup(),
            self.verified_backup(),
            self.commit(),
        ]
    }
}
