use super::super::continuity::checksums_match;
use super::{PartialRangeStore, SingleResponseState, StagedPrefix};
use crate::partial_range_manifest::RangeManifest;
use anyhow::Result;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Exposure {
    Pending,
    NewRevision,
    Continuous,
}

enum CanonicalPrefix {
    Empty,
    Probe(RangeManifest),
    Unavailable,
}

impl Exposure {
    pub(super) fn prefix(self, manifest: RangeManifest) -> StagedPrefix {
        match self {
            Self::Pending => StagedPrefix::Pending(manifest),
            Self::NewRevision | Self::Continuous => StagedPrefix::Readable(manifest),
        }
    }
}

impl PartialRangeStore {
    pub(super) async fn staged_prefix_exposure(
        &self,
        state: &SingleResponseState,
        prefix: &RangeManifest,
    ) -> Result<Exposure> {
        if state.readable_prefix().is_some() {
            return Ok(Exposure::Continuous);
        }
        let key = state.identity.post().as_str();
        if self.sparse_response_for_post(key).await
            || self.transient_responses.lock().await.contains_key(key)
        {
            return Ok(Exposure::Pending);
        }
        match self.canonical_prefix(key).await {
            CanonicalPrefix::Empty => Ok(Exposure::NewRevision),
            CanonicalPrefix::Unavailable => Ok(Exposure::Pending),
            CanonicalPrefix::Probe(canonical) => {
                self.expose_matching_probe(key, prefix, &canonical).await
            }
        }
    }

    async fn canonical_prefix(&self, key: &str) -> CanonicalPrefix {
        let entries = self.entries.lock().await;
        match entries.get(key) {
            None => CanonicalPrefix::Empty,
            Some(entry) if entry.accounted == 0 => CanonicalPrefix::Empty,
            Some(entry) if entry.completion.is_some() || entry.manifest.is_complete() => {
                CanonicalPrefix::Unavailable
            }
            Some(entry) => CanonicalPrefix::Probe(entry.manifest.clone()),
        }
    }

    async fn expose_matching_probe(
        &self,
        key: &str,
        prefix: &RangeManifest,
        canonical: &RangeManifest,
    ) -> Result<Exposure> {
        if !covered_probe(prefix, canonical) {
            return Ok(Exposure::Pending);
        }
        let path = self.paths.single_response(key);
        if checksums_match(&path, canonical.checksum_records()).await? {
            Ok(Exposure::Continuous)
        } else {
            Ok(Exposure::Pending)
        }
    }
}

fn covered_probe(prefix: &RangeManifest, canonical: &RangeManifest) -> bool {
    !canonical.checksum_records().is_empty()
        && prefix.total_len() == canonical.total_len()
        && canonical
            .ranges()
            .iter()
            .all(|range| prefix.contains(range))
}
