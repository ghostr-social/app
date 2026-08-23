use crate::partial_range_disk::{self as disk, Entry};
use crate::partial_range_manifest::IntervalChecksum;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_paths::StorePaths;
use anyhow::{ensure, Context, Result};
use std::ops::Range;
use std::path::PathBuf;

pub(super) struct ReadPlan {
    path: PathBuf,
    requested: Range<u64>,
    envelope: Range<u64>,
    checksums: Vec<IntervalChecksum>,
    stable_manifest: RangeManifest,
}

pub(super) struct ReadOutcome {
    pub(super) bytes: Vec<u8>,
    pub(super) valid: bool,
}

pub(super) enum RetryOutcome {
    Verified(Vec<u8>),
    StructuralLoss,
    Transient(anyhow::Error),
}

impl ReadPlan {
    #[cfg(test)]
    pub(super) fn capture(
        paths: &StorePaths,
        key: &str,
        entry: &Entry,
        requested: Range<u64>,
    ) -> Result<Option<Self>> {
        Self::capture_with_manifest(paths, key, entry, &entry.manifest, requested)
    }

    pub(super) fn capture_with_manifest(
        paths: &StorePaths,
        key: &str,
        entry: &Entry,
        readable: &RangeManifest,
        requested: Range<u64>,
    ) -> Result<Option<Self>> {
        if requested.start >= requested.end {
            return Ok(None);
        }
        if !readable.contains(&requested) {
            return Ok(None);
        }
        let checksums = readable.checksums_for(&requested)?;
        let envelope = checksum_envelope(&checksums).unwrap_or_else(|| requested.clone());
        let path = match entry.completion {
            Some(_) => paths.completed(key),
            None => paths.partial(key),
        };
        Ok(Some(Self {
            path,
            requested,
            envelope,
            checksums,
            stable_manifest: entry.manifest.clone(),
        }))
    }

    pub(super) fn capture_session(
        paths: &StorePaths,
        key: &str,
        manifest: &RangeManifest,
        requested: Range<u64>,
    ) -> Result<Option<Self>> {
        capture(paths.single_response(key), manifest, requested)
    }

    pub(super) async fn execute(&self) -> Result<ReadOutcome> {
        let envelope = disk::read_span(&self.path, &self.envelope).await?;
        let valid = self.checksums.iter().try_fold(true, |valid, checksum| {
            Ok::<_, anyhow::Error>(valid && self.matches(checksum, &envelope)?)
        })?;
        let bytes = slice(&envelope, &self.envelope, &self.requested)?.to_vec();
        Ok(ReadOutcome { bytes, valid })
    }

    pub(super) fn is_current(&self, paths: &StorePaths, key: &str, entry: &Entry) -> Result<bool> {
        let current_path = match entry.completion {
            Some(_) => paths.completed(key),
            None => paths.partial(key),
        };
        if current_path != self.path || entry.manifest != self.stable_manifest {
            return Ok(false);
        }
        Ok(true)
    }

    fn matches(&self, checksum: &IntervalChecksum, bytes: &[u8]) -> Result<bool> {
        let interval = slice(bytes, &self.envelope, &checksum.span())?;
        Ok(disk::sha256_bytes(interval) == checksum.digest())
    }
}

fn capture(
    path: PathBuf,
    manifest: &RangeManifest,
    requested: Range<u64>,
) -> Result<Option<ReadPlan>> {
    if requested.start >= requested.end || !manifest.contains(&requested) {
        return Ok(None);
    }
    let checksums = manifest.checksums_for(&requested)?;
    let envelope = checksum_envelope(&checksums).unwrap_or_else(|| requested.clone());
    Ok(Some(ReadPlan {
        path,
        requested,
        envelope,
        checksums,
        stable_manifest: manifest.clone(),
    }))
}

fn checksum_envelope(checksums: &[IntervalChecksum]) -> Option<Range<u64>> {
    Some(checksums.first()?.span().start..checksums.last()?.span().end)
}

fn slice<'a>(bytes: &'a [u8], envelope: &Range<u64>, span: &Range<u64>) -> Result<&'a [u8]> {
    ensure!(
        envelope.start <= span.start && span.end <= envelope.end,
        "verified range exceeds envelope"
    );
    let start = usize::try_from(span.start - envelope.start)
        .context("verified range offset exceeds memory")?;
    let end =
        usize::try_from(span.end - envelope.start).context("verified range end exceeds memory")?;
    bytes
        .get(start..end)
        .context("verified range exceeds envelope")
}

pub(super) fn verified_bytes(outcome: Result<ReadOutcome>) -> Option<Vec<u8>> {
    outcome
        .ok()
        .and_then(|outcome| outcome.valid.then_some(outcome.bytes))
}

pub(super) fn classify_retry(outcome: Result<ReadOutcome>) -> RetryOutcome {
    match outcome {
        Ok(outcome) if outcome.valid => RetryOutcome::Verified(outcome.bytes),
        Ok(_) => RetryOutcome::StructuralLoss,
        Err(error) if proves_structural_loss(&error) => RetryOutcome::StructuralLoss,
        Err(error) => RetryOutcome::Transient(error),
    }
}

fn proves_structural_loss(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause.downcast_ref::<std::io::Error>().is_some_and(|io| {
            matches!(
                io.kind(),
                std::io::ErrorKind::NotFound
                    | std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::IsADirectory
            )
        })
    })
}
