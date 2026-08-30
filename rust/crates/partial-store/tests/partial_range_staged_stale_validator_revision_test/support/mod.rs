use crate::partial_range_store::{ContentRevision, PartialRangeStore};
use ghostr_engine::representation::{HttpGenerationLease, TransferIdentity};
use std::path::PathBuf;

mod publication;
mod setup;
pub use setup::seeded_fixture;

pub const LONG_BODY: &[u8] = b"old! data";
pub const NEW_BODY: &[u8] = b"new data";
pub(super) const OLD_PREFIX: &[u8] = b"old!";
pub(super) const SOURCE: &str = "https://cdn.example/clip.mp4";
pub(super) const STRONG_ETAG: &str = "\"stable\"";

pub struct Fixture {
    pub(super) root: PathBuf,
    pub(super) store: PartialRangeStore,
    pub(super) transfer: TransferIdentity,
    pub(super) lease: HttpGenerationLease,
}

impl Fixture {
    pub async fn revision(&self) -> ContentRevision {
        self.store
            .media_snapshot("clip")
            .await
            .expect("snapshot")
            .revision()
    }

    pub async fn current_bytes(&self, length: usize) -> Vec<u8> {
        self.store
            .read_range("clip", 0..length as u64)
            .await
            .expect("current body read")
            .expect("current body")
    }

    pub fn cleanup(self) {
        let root = self.root.clone();
        drop(self);
        crate::tests::store_fixture::discard(&root);
    }
}
