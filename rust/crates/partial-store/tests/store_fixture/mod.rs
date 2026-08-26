//! Test stores use either a temp root or controllable free space.

use crate::partial_range_store::PartialRangeStore;
use ghostr_engine::representation::{
    HttpGenerationAuthority, RepresentationBinding, SourceGeneration, TransferIdentity,
};
use std::path::{Path, PathBuf};

mod contracts;
pub(super) use contracts::exact_response;
mod http_generation;
mod paths;
mod response_commit;
mod response_mode;
mod space;
pub(super) use space::{
    limits, paced_store, plain_store, reopened, spaced_store, FakeSpace, SpacedStore,
};
mod whole;

pub(super) fn http_generation(final_url: &str, etag: &str, epoch: u64) -> HttpGenerationAuthority {
    http_generation::http_generation(final_url, etag, epoch)
}

pub(super) async fn staged_replacement(
    prefix: &str,
) -> (PathBuf, PartialRangeStore, RepresentationBinding) {
    response_commit::staged_replacement(prefix).await
}

pub(super) async fn backup_canonical(root: &Path) {
    response_commit::backup_canonical(root).await;
}

pub(super) fn response_commit(phase: &str) -> String {
    response_commit::response_commit(phase)
}

pub(super) async fn mode_fixture(prefix: &str) -> (PathBuf, PartialRangeStore, TransferIdentity) {
    response_mode::mode_fixture(prefix).await
}

pub(super) fn source_generation() -> SourceGeneration {
    response_mode::source_generation()
}

pub(super) async fn publish_whole(
    store: &PartialRangeStore,
    identity: &TransferIdentity,
    id: u64,
    bytes: &[u8],
) {
    whole::publish_whole(store, identity, id, bytes).await;
}

pub(super) fn discard(root: &Path) {
    paths::discard(root);
}

pub(super) fn temp_root(prefix: &str) -> PathBuf {
    paths::temp_root(prefix)
}
