use crate::api::delivery::snapshots::{
    error_event, event_for, DeliverySnapshot, DeliverySnapshotAuthority,
};
use crate::api::delivery_types::FfiDeliveryEventKind;

fn snapshot(startable: bool, bytes: u64) -> DeliverySnapshot {
    DeliverySnapshot {
        startable,
        bytes_present: bytes,
        total_bytes: Some(16),
        eta_ms: None,
        failed: false,
        detail: None,
        authority: None,
        hls_authority: None,
    }
}

#[test]
fn the_first_observation_reports_readiness() {
    let event = event_for("clip", None, snapshot(false, 0)).expect("first event");

    assert_eq!(event.kind, FfiDeliveryEventKind::Readiness);
    assert_eq!(event.eta_ms, None);
    assert_eq!(event.post_id, "clip");
    assert!(!event.startable);
}

#[test]
fn a_startability_flip_reports_readiness() {
    let previous = snapshot(false, 8);

    let event = event_for("clip", Some(&previous), snapshot(true, 16)).expect("flip event");

    assert_eq!(event.kind, FfiDeliveryEventKind::Readiness);
    assert!(event.startable);
    assert_eq!(event.bytes_present, 16);
}

#[test]
fn byte_growth_alone_reports_progress() {
    let previous = snapshot(false, 8);

    let event = event_for("clip", Some(&previous), snapshot(false, 12)).expect("progress event");

    assert_eq!(event.kind, FfiDeliveryEventKind::Progress);
    assert_eq!(event.bytes_present, 12);
    assert_eq!(event.total_bytes, Some(16));
}

#[test]
fn an_unchanged_snapshot_stays_silent() {
    let previous = snapshot(true, 16);

    assert!(event_for("clip", Some(&previous), snapshot(true, 16)).is_none());
}

#[test]
fn error_events_carry_the_detail() {
    let event = error_event("clip", "store failed".to_owned());

    assert_eq!(event.kind, FfiDeliveryEventKind::Error);
    assert_eq!(event.post_id, "clip");
    assert!(!event.startable);
    assert_eq!(event.detail.as_deref(), Some("store failed"));
}

#[test]
fn a_playback_block_is_distinct_from_an_observer_error() {
    let mut failed = snapshot(false, 8);
    failed.failed = true;
    failed.detail = Some("decoder unsupported".to_owned());
    failed.authority = Some(DeliverySnapshotAuthority {
        representation_id: "a".repeat(64),
        asset_id: "b".repeat(43),
    });

    let event = event_for("clip", None, failed).expect("failure event");

    assert_eq!(event.kind, FfiDeliveryEventKind::Failed);
    assert_eq!(event.detail.as_deref(), Some("decoder unsupported"));
    assert_eq!(event.representation_id, Some("a".repeat(64)));
    assert_eq!(event.asset_id, Some("b".repeat(43)));
}
