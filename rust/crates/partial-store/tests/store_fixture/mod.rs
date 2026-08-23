#![allow(dead_code)]
//! Test stores use either a temp root or controllable free space.

use std::path::{Path, PathBuf};

mod contracts;
pub use contracts::exact_response;
mod http_generation;
#[allow(unused_imports)]
pub use http_generation::http_generation;
mod paths;
mod response_commit;
#[allow(unused_imports)]
pub use response_commit::{backup_canonical, response_commit, staged_replacement};
mod response_mode;
#[allow(unused_imports)]
pub use response_mode::{mode_fixture, source_generation};
mod space;
#[allow(unused_imports)]
pub use space::{limits, paced_store, plain_store, reopened, spaced_store, FakeSpace, SpacedStore};
mod whole;
#[allow(unused_imports)]
pub use whole::publish_whole;

pub fn discard(root: &Path) {
    paths::discard(root);
}

pub fn temp_root(prefix: &str) -> PathBuf {
    paths::temp_root(prefix)
}
