use crate::api::delivery::snapshots::{compute_snapshot, SnapshotInput};
use crate::api::tests::support::sized_meta;
use crate::engine::{ByteRange, EngineParams, PostId};

#[test]
fn becomes_startable_once_the_head_is_on_disk() {
    let meta = sized_meta(16, 2_000);
    let params = EngineParams::default();
    let post = PostId::new("clip");
    let covered = [ByteRange::new(0, 16)];

    let snapshot = compute_snapshot(
        &post,
        SnapshotInput {
            meta: &meta,
            ranges: &covered,
            stored_total: None,
            params: &params,
        },
    );

    assert!(snapshot.startable);
    assert_eq!(snapshot.bytes_present, 16);
    assert_eq!(snapshot.total_bytes, Some(16));
}

#[test]
fn stays_unstartable_while_head_bytes_are_missing() {
    let meta = sized_meta(16, 2_000);
    let params = EngineParams::default();
    let post = PostId::new("clip");
    let partial = [ByteRange::new(0, 8)];

    let snapshot = compute_snapshot(
        &post,
        SnapshotInput {
            meta: &meta,
            ranges: &partial,
            stored_total: None,
            params: &params,
        },
    );

    assert!(!snapshot.startable);
    assert_eq!(snapshot.bytes_present, 8);
}

#[test]
fn a_stored_total_beats_the_discovery_size() {
    let meta = sized_meta(16, 2_000);
    let params = EngineParams::default();
    let post = PostId::new("clip");

    let snapshot = compute_snapshot(
        &post,
        SnapshotInput {
            meta: &meta,
            ranges: &[],
            stored_total: Some(20),
            params: &params,
        },
    );

    assert_eq!(snapshot.total_bytes, Some(20));
}
