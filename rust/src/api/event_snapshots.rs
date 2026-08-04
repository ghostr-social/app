//! Per-post delivery snapshots and their event diffs. Pure logic —
//! the watcher feeds it store reads, tests feed it tables.

use crate::api::delivery_types::{FfiDeliveryEvent, FfiDeliveryEventKind};
use crate::engine::catalog::{Catalog, LearnedFacts};
use crate::engine::inventory_controller::is_startable;
use crate::engine::{ByteRange, EngineParams, PostId, VideoMeta};

/// What one tracked post looks like right now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeliverySnapshot {
    pub startable: bool,
    pub bytes_present: u64,
    pub total_bytes: Option<u64>,
}

/// Everything a snapshot is computed from.
pub(crate) struct SnapshotInput<'a> {
    pub meta: &'a VideoMeta,
    pub ranges: &'a [ByteRange],
    /// The store's declared total; beats the discovery size.
    pub stored_total: Option<u64>,
    pub params: &'a EngineParams,
}

pub(crate) fn compute_snapshot(post: &PostId, input: SnapshotInput<'_>) -> DeliverySnapshot {
    let catalog = catalog_for(post, input.meta, input.stored_total);
    DeliverySnapshot {
        startable: is_startable(&catalog, post, input.ranges, input.params),
        bytes_present: input.ranges.iter().map(ByteRange::len).sum(),
        total_bytes: input.stored_total.or(input.meta.size_bytes),
    }
}

/// A single-post catalog so the engine's startability rule (head on
/// disk, moov reachable) is reused instead of re-derived here.
fn catalog_for(post: &PostId, meta: &VideoMeta, stored_total: Option<u64>) -> Catalog {
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta.clone());
    if let Some(total) = stored_total {
        let facts = LearnedFacts {
            content_length: Some(total),
            ..LearnedFacts::default()
        };
        catalog.learn(post, facts);
    }
    catalog
}

/// The event a state transition produces; `None` when nothing changed.
pub(crate) fn event_for(
    post_id: &str,
    previous: Option<&DeliverySnapshot>,
    current: DeliverySnapshot,
) -> Option<FfiDeliveryEvent> {
    let kind = change_kind(previous, &current)?;
    Some(FfiDeliveryEvent {
        post_id: post_id.to_owned(),
        kind,
        startable: current.startable,
        bytes_present: current.bytes_present,
        total_bytes: current.total_bytes,
        detail: None,
    })
}

fn change_kind(
    previous: Option<&DeliverySnapshot>,
    current: &DeliverySnapshot,
) -> Option<FfiDeliveryEventKind> {
    match previous {
        None => Some(FfiDeliveryEventKind::Readiness),
        Some(prev) if prev.startable != current.startable => Some(FfiDeliveryEventKind::Readiness),
        Some(prev) if prev != current => Some(FfiDeliveryEventKind::Progress),
        Some(_) => None,
    }
}

pub(crate) fn error_event(post_id: &str, detail: String) -> FfiDeliveryEvent {
    FfiDeliveryEvent {
        post_id: post_id.to_owned(),
        kind: FfiDeliveryEventKind::Error,
        startable: false,
        bytes_present: 0,
        total_bytes: None,
        detail: Some(detail),
    }
}
