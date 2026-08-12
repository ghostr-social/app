use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, PostId};

const TOTAL: u64 = 370_912;
const PLAYBACK_SLICE: u64 = 256 * 1_024;

#[test]
fn protected_posts_get_one_bounded_bootstrap_grant_before_depth() {
    let mut bench = WorkBench::new();
    for post in ["current", "next-1", "next-2", "next-3"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(TOTAL), Some(3_000)),
        );
    }
    bench.focus = focus_at(&["current", "next-1", "next-2", "next-3"], 0, 0);

    let first: Vec<_> = bench
        .run()
        .into_iter()
        .take(4)
        .map(|work| (work.chunk.post, work.chunk.range))
        .collect();

    assert_eq!(
        first,
        [
            (PostId::new("current"), ByteRange::new(0, TOTAL)),
            (PostId::new("next-1"), ByteRange::new(0, PLAYBACK_SLICE)),
            (PostId::new("next-2"), ByteRange::new(0, PLAYBACK_SLICE)),
            (PostId::new("next-3"), ByteRange::new(0, PLAYBACK_SLICE)),
        ]
    );
}
