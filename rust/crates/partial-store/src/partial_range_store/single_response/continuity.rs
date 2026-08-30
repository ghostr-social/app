use super::{PartialRangeStore, SingleResponseState};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::IntervalChecksum;
use anyhow::Result;
use ghostr_engine::representation::SourceGeneration;
use std::path::Path;

#[derive(Clone, Copy)]
pub(super) struct StagedCommitPolicy {
    pub(super) preserve_revision: bool,
    pub(super) retire_http: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SparseContinuity {
    Unproven,
    Confirmed,
    Conflicted,
}

impl PartialRangeStore {
    pub(super) async fn staged_commit_policy(
        &self,
        state: &SingleResponseState,
        total: u64,
    ) -> Result<StagedCommitPolicy> {
        let continuity = self.durable_response_continuity(state, total).await?;
        Ok(StagedCommitPolicy {
            preserve_revision: continuity == SparseContinuity::Confirmed,
            retire_http: state.authority.retires_http_generation()
                || continuity == SparseContinuity::Conflicted,
        })
    }

    async fn durable_response_continuity(
        &self,
        state: &SingleResponseState,
        total: u64,
    ) -> Result<SparseContinuity> {
        let Some(generation) = self.current_sparse_generation(state).await else {
            return Ok(SparseContinuity::Unproven);
        };
        if !self
            .http_generation_matches_source(&state.identity, &generation)
            .await
        {
            return Ok(SparseContinuity::Unproven);
        }
        if generation.total_bytes() != total {
            return Ok(SparseContinuity::Conflicted);
        }
        let key = state.identity.post().as_str();
        let checksums = self.stable_checksums(key).await?;
        if checksums.is_empty() {
            return Ok(SparseContinuity::Unproven);
        }
        match checksums_match(&self.paths.single_response(key), &checksums).await? {
            true => Ok(SparseContinuity::Confirmed),
            false => Ok(SparseContinuity::Conflicted),
        }
    }

    async fn current_sparse_generation(
        &self,
        state: &SingleResponseState,
    ) -> Option<SourceGeneration> {
        let lease = state.authority.generation()?;
        if !self
            .http_generation_is_current(&state.identity, lease)
            .await
        {
            return None;
        }
        self.generation_for(
            state.identity.post().as_str(),
            state.identity.source().as_str(),
        )
        .await
    }

    async fn stable_checksums(&self, key: &str) -> Result<Vec<IntervalChecksum>> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(entry.manifest.checksum_records().to_vec())
    }
}

async fn checksums_match(path: &Path, checksums: &[IntervalChecksum]) -> Result<bool> {
    for checksum in checksums {
        let observed = disk::sha256_span(path, &checksum.span()).await?;
        if observed != checksum.digest() {
            return Ok(false);
        }
    }
    Ok(true)
}
