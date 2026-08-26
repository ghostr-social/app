use crate::api::delivery::snapshots::{compute_snapshot, SnapshotInput};
use crate::api::tests::support::sized_meta;
use crate::engine::{ByteRange, EngineParams, PostId};

#[test]
fn cached_decoder_blocked_representation_is_failed_not_startable() {
    let meta = sized_meta(16, 2_000);
    let params = EngineParams::default();
    let covered = [ByteRange::new(0, 16)];

    let snapshot = compute_snapshot(
        &PostId::new("clip"),
        SnapshotInput {
            meta: &meta,
            ranges: &covered,
            stored_total: None,
            params: &params,
            playback_blocked: true,
            authority: None,
        },
    );

    assert!(snapshot.failed);
    assert!(!snapshot.startable);
    assert_eq!(snapshot.detail.as_deref(), Some("decoder unsupported"));
}
