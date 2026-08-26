use super::{PartialRangeStore, SessionResponse};
use crate::partial_range_completion::Completion;
use crate::partial_range_store::cleanup_debt::CleanupScope;
use crate::partial_range_store::replacement_cleanup;
use crate::partial_range_store::single_response::{transaction, ResponseCommit};
use crate::partial_range_store::Entries;
use anyhow::{Context as _, Result};

pub(super) async fn publish(
    store: &PartialRangeStore,
    entries: &mut Entries,
    key: &str,
    response: &SessionResponse,
    digest: String,
) -> Result<Completion> {
    let old = entries.get(key).context("session entry present")?.accounted;
    let mut record = ResponseCommit::verified(response.bytes(), digest);
    if let Err(error) = transaction::publish(&store.paths, key, &mut record).await {
        transaction::rollback_commit(&store.paths, key, &record, true)
            .await
            .with_context(|| format!("rollback verified response after: {error:#}"))?;
        return Err(error);
    }
    record_entry(store, entries, key, response).await;
    let pending = store.take_sparse_response_bytes(key).await;
    finish(store, key, old.saturating_add(pending)).await?;
    Ok(Completion::Verified)
}

async fn record_entry(
    store: &PartialRangeStore,
    entries: &mut Entries,
    key: &str,
    response: &SessionResponse,
) {
    let entry = entries.get_mut(key).expect("session entry present");
    entry.manifest = response.manifest().clone();
    entry.accounted = response.bytes();
    entry.completion = Some(Completion::Verified);
    entry.touched = store.tick();
    store.take_session_response(key).await;
    store.advance_content_revision(key).await;
    store.retire_generation(key).await;
    store.retire_http_generation(key).await;
    store.changed.notify_waiters();
}

async fn finish(store: &PartialRangeStore, key: &str, replaced: u64) -> Result<()> {
    if let Err(error) = replacement_cleanup::published(&store.paths, key).await {
        store
            .transfer_charged_cleanup_debt(key, CleanupScope::ReplacedCanonical, replaced)
            .await?;
        log::warn!("Could not clean replaced verified video {key}: {error:#}");
        return Ok(());
    }
    store.release(replaced).await;
    Ok(())
}
