//! Per-post delivery snapshots and their event diffs. Pure logic —
//! the watcher feeds it store reads, tests feed it tables.

use crate::api::delivery_types::{FfiDeliveryEvent, FfiDeliveryEventKind};
use crate::engine::adaptive::{candidate_snapshot, CandidateEvidence, FeedOffset, ViewProbability};
use crate::engine::catalog::{Catalog, LearnedFacts};
use crate::engine::{ByteRange, EngineParams, PostId, VideoMeta};
use ghostr_delivery::segmented::{SegmentedPhase, SegmentedSnapshot};

/// What one tracked post looks like right now.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeliverySnapshot {
    pub startable: bool,
    pub bytes_present: u64,
    pub total_bytes: Option<u64>,
    pub eta_ms: Option<u64>,
    pub failed: bool,
    pub detail: Option<String>,
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
        startable: initial_playable_range_is_cached(&catalog, post, &input),
        bytes_present: input.ranges.iter().map(ByteRange::len).sum(),
        total_bytes: input.stored_total.or(input.meta.size_bytes),
        eta_ms: None,
        failed: false,
        detail: None,
    }
}

pub(crate) fn hls_snapshot(snapshot: SegmentedSnapshot) -> DeliverySnapshot {
    DeliverySnapshot {
        startable: snapshot.phase == SegmentedPhase::Ready,
        bytes_present: snapshot.bytes_present,
        total_bytes: None,
        eta_ms: snapshot.eta_ms,
        failed: snapshot.phase == SegmentedPhase::Failed,
        detail: snapshot.detail,
    }
}

fn initial_playable_range_is_cached(
    catalog: &Catalog,
    post: &PostId,
    input: &SnapshotInput<'_>,
) -> bool {
    let evidence = CandidateEvidence {
        post: post.clone(),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid probability"),
        present: input.ranges.to_vec(),
        recently_evicted: Vec::new(),
        in_flight: Vec::new(),
        origins: Vec::new(),
    };
    candidate_snapshot(catalog, input.params, evidence)
        .and_then(|candidate| candidate.playable_ranges.into_iter().next())
        .is_some_and(|playable| covers(input.ranges, playable.bytes))
}

fn covers(ranges: &[ByteRange], wanted: ByteRange) -> bool {
    crate::engine::media_timeline::normalize(ranges.to_vec())
        .iter()
        .any(|range| range.start <= wanted.start && range.end >= wanted.end)
}

/// A single-post catalog so adaptive playable-range geometry is reused.
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
        eta_ms: current.eta_ms,
        detail: current.detail.clone(),
    })
}

fn change_kind(
    previous: Option<&DeliverySnapshot>,
    current: &DeliverySnapshot,
) -> Option<FfiDeliveryEventKind> {
    if current.failed {
        return Some(FfiDeliveryEventKind::Error);
    }
    match previous {
        None => Some(FfiDeliveryEventKind::Readiness),
        Some(previous) => changed_snapshot(previous, current),
    }
}

fn changed_snapshot(
    previous: &DeliverySnapshot,
    current: &DeliverySnapshot,
) -> Option<FfiDeliveryEventKind> {
    if previous == current {
        return None;
    }
    if current.failed {
        return Some(FfiDeliveryEventKind::Error);
    }
    if previous.startable != current.startable {
        return Some(FfiDeliveryEventKind::Readiness);
    }
    Some(FfiDeliveryEventKind::Progress)
}

pub(crate) fn error_event(post_id: &str, detail: String) -> FfiDeliveryEvent {
    FfiDeliveryEvent {
        post_id: post_id.to_owned(),
        kind: FfiDeliveryEventKind::Error,
        startable: false,
        bytes_present: 0,
        total_bytes: None,
        eta_ms: None,
        detail: Some(detail),
    }
}
