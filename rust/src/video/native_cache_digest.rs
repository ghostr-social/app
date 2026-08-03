use crate::video::native_cache_failure::permanent;
use crate::video::native_cache_fetch::FetchedVideo;
use crate::video::native_partial_store::NativePartialStore;
use anyhow::Result;
use std::path::Path;

pub async fn verify_digest(
    partials: &NativePartialStore,
    partial: &Path,
    expected: Option<&str>,
    fetched: &FetchedVideo,
) -> Result<()> {
    if expected.is_none_or(|value| value.eq_ignore_ascii_case(&fetched.sha256)) {
        return Ok(());
    }
    let error = permanent("downloaded video digest does not match advertised digest");
    Err(partials.cleanup_error(partial, fetched.bytes, error).await)
}
