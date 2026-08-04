use crate::engine::catalog::Catalog;
use crate::engine::inventory_controller::is_startable;
use crate::engine::tests::support::progressive_meta;
use crate::engine::{ByteRange, EngineParams, PostId};

const KIB: u64 = 1024;
const MIB: u64 = 1024 * 1024;

fn startable(size: Option<u64>, duration: Option<u64>, have: &[ByteRange]) -> bool {
    let post = PostId::new("a");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), progressive_meta(size, duration));
    is_startable(&catalog, &post, have, &EngineParams::default())
}

#[test]
fn a_full_head_with_known_duration_is_startable() {
    // Measured bitrate is high, so the head caps at 3 MiB.
    let have = [ByteRange::new(0, 3 * MIB)];

    assert!(startable(Some(10 * MIB), Some(8_000), &have));
}

#[test]
fn a_partial_head_is_not_startable() {
    let have = [ByteRange::new(0, 2 * MIB)];

    assert!(!startable(Some(10 * MIB), Some(8_000), &have));
}

#[test]
fn split_ranges_that_union_to_the_head_are_startable() {
    let have = [
        ByteRange::new(2 * MIB - KIB, 3 * MIB),
        ByteRange::new(0, 2 * MIB),
    ];

    assert!(startable(Some(10 * MIB), Some(8_000), &have));
}

#[test]
fn unknown_duration_leaves_moov_pending_until_the_tail_probe_lands() {
    // Assumed bitrate → head 1_250_000; the probe is the last 256 KiB.
    let head_only = [ByteRange::new(0, 1_250_000)];
    let with_probe = [
        ByteRange::new(0, 1_250_000),
        ByteRange::new(10 * MIB - 256 * KIB, 10 * MIB),
    ];

    assert!(!startable(Some(10 * MIB), None, &head_only));
    assert!(startable(Some(10 * MIB), None, &with_probe));
}

#[test]
fn unknown_duration_and_size_can_never_be_startable_yet() {
    assert!(!startable(None, None, &[ByteRange::new(0, 100 * MIB)]));
}

#[test]
fn a_post_missing_from_the_catalog_is_not_startable() {
    let catalog = Catalog::new();
    let post = PostId::new("missing");

    assert!(!is_startable(
        &catalog,
        &post,
        &[],
        &EngineParams::default()
    ));
}
