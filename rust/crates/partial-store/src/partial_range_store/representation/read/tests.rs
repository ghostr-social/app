use super::{ContentRevision, RepresentationRead};
use crate::partial_range_store::capacity::StoreCapacity;
use crate::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[tokio::test]
async fn an_error_from_a_replaced_stream_is_reported_as_superseded() {
    let store = store();
    let revision = ContentRevision::default();
    store.advance_content_revision("clip").await;

    let read = Err(anyhow::anyhow!("stale read failure"));
    assert!(matches!(
        store
            .finish_stream_read("clip", None, revision, read)
            .await
            .unwrap(),
        RepresentationRead::Superseded
    ));
}

fn store() -> PartialRangeStore {
    let root = std::env::temp_dir().join(format!(
        "ghostr-stale-read-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    PartialRangeStore::with_capacity(
        root,
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    )
}
