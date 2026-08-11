use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, PostId};

pub(super) const POSTS: &[&str] = &["current", "next1", "next2", "next3", "next4", "next5"];
const HEAD_END: u64 = 2_000_000;

pub(super) fn bench() -> WorkBench {
    let mut bench = WorkBench::new();
    for post in POSTS {
        bench.catalog.upsert(
            PostId::new(*post),
            progressive_meta(Some(8_000_000), Some(16_000)),
        );
    }
    bench.focus = focus_at(POSTS, 0, 0);
    bench
}

pub(super) fn mark_startable(bench: &mut WorkBench, posts: &[&str]) {
    for post in posts {
        bench
            .present
            .insert(PostId::new(*post), vec![ByteRange::new(0, HEAD_END)]);
    }
}
