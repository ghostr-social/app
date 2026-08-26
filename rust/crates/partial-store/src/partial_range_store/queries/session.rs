use super::evidence::StoredEvidence;
use super::read::{self, ReadPlan, RetryOutcome};
use super::snapshot::StoredMediaSnapshot;
use crate::partial_range_store::single_response::SessionResponse;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;
use core::ops::Range;

pub(super) async fn snapshot(
    store: &PartialRangeStore,
    key: &str,
    response: &SessionResponse,
) -> Result<StoredMediaSnapshot> {
    let manifest = response.manifest();
    let ranges = manifest.ranges();
    Ok(StoredMediaSnapshot {
        binding: current_binding(store, key, response).await,
        revision: store.current_content_revision(key).await,
        total_len: manifest.total_len(),
        ranges: ranges.clone(),
        planning_ranges: ranges,
        complete: true,
        finalized: false,
        continuation_source: None,
        evidence: StoredEvidence::capture(manifest),
    })
}

async fn current_binding(
    store: &PartialRangeStore,
    key: &str,
    response: &SessionResponse,
) -> Option<ghostr_engine::representation::RepresentationBinding> {
    store.representation_binding(key).await.filter(|binding| {
        binding
            .transfer(response.identity().source().as_str())
            .as_ref()
            == Some(response.identity())
    })
}

pub(super) async fn read(
    store: &PartialRangeStore,
    key: &str,
    response: &SessionResponse,
    span: Range<u64>,
) -> Result<Option<Vec<u8>>> {
    touch(store, key).await?;
    let Some(plan) = ReadPlan::capture_session(&store.paths, key, response.manifest(), span)?
    else {
        return Ok(None);
    };
    if let Some(bytes) = read::verified_bytes(plan.execute().await) {
        return Ok(Some(bytes));
    }
    match read::classify_retry(plan.execute().await) {
        RetryOutcome::Verified(bytes) => Ok(Some(bytes)),
        RetryOutcome::Transient(error) => Err(error),
        RetryOutcome::StructuralLoss => {
            store.discard_session_response(key).await?;
            Ok(None)
        }
    }
}

async fn touch(store: &PartialRangeStore, key: &str) -> Result<()> {
    let mut entries = store.entries.lock().await;
    let touched = store.tick();
    store.entry(&mut entries, key).await?.touched = touched;
    Ok(())
}
