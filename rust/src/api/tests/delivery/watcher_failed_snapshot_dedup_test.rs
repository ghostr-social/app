use crate::api::delivery::snapshots::{event_for, DeliverySnapshot};
use crate::api::delivery_types::FfiDeliveryEventKind;

#[test]
fn failed_snapshots_emit_once_until_the_failure_changes() {
    let failed = DeliverySnapshot {
        startable: false,
        bytes_present: 0,
        total_bytes: None,
        eta_ms: None,
        failed: true,
        detail: Some("all sources failed".to_owned()),
        authority: None,
        hls_authority: None,
    };
    assert_eq!(
        event_for("clip", None, failed.clone())
            .expect("initial failure")
            .kind,
        FfiDeliveryEventKind::Failed
    );
    assert!(
        event_for("clip", Some(&failed), failed.clone()).is_none(),
        "unrelated store changes must not replay unchanged failures"
    );
    let changed = DeliverySnapshot {
        detail: Some("decoder unsupported".to_owned()),
        ..failed.clone()
    };
    assert_eq!(
        event_for("clip", Some(&failed), changed)
            .expect("changed failure")
            .kind,
        FfiDeliveryEventKind::Failed
    );
}
