//! Whether a byte-complete file may leave the partial pool, and what
//! its bytes are worth as evidence once it has.

use crate::video::partial_range_disk as disk;
use anyhow::Result;
use std::path::Path;

/// How a completed file's bytes were checked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Completion {
    /// The stored bytes hashed to the digest the note advertised.
    Verified,
    /// The note advertised no digest, so there was nothing to check the
    /// bytes against: complete, but not attested.
    Unverified,
}

impl Completion {
    /// True only for bytes that were hashed and matched. Callers that
    /// need integrity must ask this instead of assuming completeness.
    pub fn is_verified(self) -> bool {
        matches!(self, Self::Verified)
    }
}

/// Decides a byte-complete file's fate. Plan §8 keeps mismatched bytes
/// out of the cache, but most Nostr video notes advertise no `imeta x`
/// at all: those files are kept as [`Completion::Unverified`] instead
/// of sitting in the partial pool forever. `None` means "discard".
pub(crate) async fn judge(partial: &Path, advertised: Option<&str>) -> Result<Option<Completion>> {
    let Some(expected) = advertised else {
        return Ok(Some(Completion::Unverified));
    };
    let digest = disk::sha256_file(partial).await?;
    Ok(digest
        .eq_ignore_ascii_case(expected)
        .then_some(Completion::Verified))
}

/// Records the verdict beside the completed file so a later run can
/// still tell a checked file from a merely finished one.
pub(crate) async fn record(marker: &Path, completion: Completion) -> Result<()> {
    match completion {
        Completion::Verified => disk::write_marker(marker).await,
        Completion::Unverified => disk::remove_if_present(marker).await,
    }
}

/// Reads a completed file's provenance back from disk. A missing marker
/// reads as unverified, so nothing is ever promoted to "verified" by
/// losing state.
pub(crate) async fn recorded(marker: &Path) -> Result<Completion> {
    let present = disk::file_len(marker).await?.is_some();
    Ok(if present {
        Completion::Verified
    } else {
        Completion::Unverified
    })
}
