use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

const VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::partial_range_store) enum CommitTarget {
    Partial,
    Verified,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::partial_range_store) enum CommitPhase {
    Prepared,
    BackedUp,
    Committed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::partial_range_store) struct ResponseCommit {
    version: u8,
    phase: CommitPhase,
    target: CommitTarget,
    total: u64,
    sha256: String,
    retire_http: bool,
}

impl ResponseCommit {
    pub(in crate::partial_range_store) fn partial(
        total: u64,
        sha256: String,
        retire_http: bool,
    ) -> Self {
        Self {
            version: VERSION,
            phase: CommitPhase::Prepared,
            target: CommitTarget::Partial,
            total,
            sha256,
            retire_http,
        }
    }

    pub(in crate::partial_range_store) fn verified(total: u64, sha256: String) -> Self {
        Self {
            version: VERSION,
            phase: CommitPhase::Prepared,
            target: CommitTarget::Verified,
            total,
            sha256,
            retire_http: true,
        }
    }

    pub(in crate::partial_range_store) fn phase(&self) -> CommitPhase {
        self.phase
    }

    pub(in crate::partial_range_store) fn target(&self) -> CommitTarget {
        self.target
    }

    pub(in crate::partial_range_store) fn total(&self) -> u64 {
        self.total
    }

    pub(in crate::partial_range_store) fn sha256(&self) -> &str {
        &self.sha256
    }

    pub(in crate::partial_range_store) fn retire_http(&self) -> bool {
        self.retire_http
    }

    pub(in crate::partial_range_store) async fn save_phase(
        &mut self,
        paths: &StorePaths,
        key: &str,
        phase: CommitPhase,
    ) -> Result<()> {
        self.phase = phase;
        let bytes = serde_json::to_vec(self).context("encode response commit")?;
        disk::save_durable(
            &paths.single_response_commit(key),
            &paths.single_response_commit_staging(key),
            &bytes,
        )
        .await
    }

    pub(in crate::partial_range_store) async fn load(
        paths: &StorePaths,
        key: &str,
    ) -> Result<Option<Self>> {
        let bytes = match tokio::fs::read(paths.single_response_commit(key)).await {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error).context("read response commit"),
        };
        let record: Self = serde_json::from_slice(&bytes).context("decode response commit")?;
        ensure!(
            record.version == VERSION,
            "unsupported response commit version"
        );
        ensure!(record.total > 0, "response commit has no bytes");
        ensure!(
            record.sha256.len() == 64,
            "response commit digest is malformed"
        );
        Ok(Some(record))
    }
}
