use ghostr_engine::media_timeline::compiled;
use ghostr_engine::representation::{RepresentationId, SourceGeneration};
use sha2::{Digest as _, Sha256};

/// Derived structure is scoped to a validated source inside this store's access partition.
/// Possessing this key grants neither access nor cross-origin byte authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledIndexKey {
    pub(super) storage: String,
    pub(super) total: u64,
}

impl CompiledIndexKey {
    /// The native MP4 compiler selects all supported audio/video tracks. Changing
    /// that selection contract or backend assumptions requires a new profile.
    pub fn native_mp4(representation: &RepresentationId, source: &SourceGeneration) -> Self {
        let mut digest = Sha256::new();
        for field in [
            "local-index-v1/all-supported-tracks/native-progressive-v1",
            compiled::PROFILE,
            representation.fingerprint(),
            source.final_url(),
            source.strong_etag(),
        ] {
            digest.update((field.len() as u64).to_be_bytes());
            digest.update(field.as_bytes());
        }
        digest.update(source.total_bytes().to_be_bytes());
        Self {
            storage: format!("{}{:x}", super::PREFIX, digest.finalize()),
            total: source.total_bytes(),
        }
    }
}
