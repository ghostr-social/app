use crate::partial_range_completion::{self as completion, Completion, IntegrityMismatch};
use crate::partial_range_store::single_response::SessionResponse;
use crate::partial_range_store::{Entries, PartialRangeStore};
use anyhow::{ensure, Result};

pub(super) async fn finalize(
    store: &PartialRangeStore,
    entries: &mut Entries,
    key: &str,
    advertised: Option<&str>,
    response: &SessionResponse,
) -> Result<Completion> {
    ensure!(
        response.is_complete(),
        "cannot finalize an active response before EOF"
    );
    let Some(advertised) = advertised else {
        return Ok(Completion::Unverified);
    };
    let path = store.paths.single_response(key);
    let Some(Completion::Verified) = completion::judge(&path, Some(advertised)).await? else {
        store.discard_session_response(key).await?;
        return Err(IntegrityMismatch.into());
    };
    store
        .promote_verified_session(entries, key, response, advertised.to_ascii_lowercase())
        .await
}
