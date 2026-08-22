use super::valid_digest;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const VERSION: u8 = 3;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::partial_range_store) struct TailIntent {
    version: u8,
    old_accounted: u64,
    new_accounted: u64,
    old_manifest_sha256: String,
    new_manifest_sha256: String,
    tail_end: u64,
}

impl TailIntent {
    pub(in crate::partial_range_store) fn new(
        old_accounted: u64,
        new_accounted: u64,
        old_manifest_sha256: String,
        new_manifest_sha256: String,
        tail_end: u64,
    ) -> Self {
        Self {
            version: VERSION,
            old_accounted,
            new_accounted,
            old_manifest_sha256,
            new_manifest_sha256,
            tail_end,
        }
    }

    pub(in crate::partial_range_store) fn validate(&self) -> Result<()> {
        ensure!(self.version == VERSION, "unsupported tail eviction intent");
        ensure!(self.old_accounted > 0, "tail intent has no old bytes");
        ensure!(self.new_accounted > 0, "tail intent has no retained bytes");
        ensure!(
            self.new_accounted < self.old_accounted,
            "tail intent does not evict bytes"
        );
        ensure!(self.tail_end > 0, "tail intent truncates the entire object");
        ensure!(
            valid_digest(&self.old_manifest_sha256),
            "invalid old tail manifest hash"
        );
        ensure!(
            valid_digest(&self.new_manifest_sha256),
            "invalid new tail manifest hash"
        );
        Ok(())
    }

    pub(in crate::partial_range_store) fn old_accounted(&self) -> u64 {
        self.old_accounted
    }

    pub(in crate::partial_range_store) fn new_accounted(&self) -> u64 {
        self.new_accounted
    }

    pub(in crate::partial_range_store) fn old_manifest_sha256(&self) -> &str {
        &self.old_manifest_sha256
    }

    pub(in crate::partial_range_store) fn new_manifest_sha256(&self) -> &str {
        &self.new_manifest_sha256
    }

    pub(in crate::partial_range_store) fn tail_end(&self) -> u64 {
        self.tail_end
    }
}
