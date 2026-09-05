use crate::tests::media_timeline_dependency_support::{tail_timeline, with_sample_table};
use crate::tests::media_timeline_support::{full_box, values};
use crate::ByteRange;

#[test]
fn continuation_window_includes_earlier_random_access_dependencies() {
    let movie = with_sample_table(&[100, 200, 300], full_box(b"stss", values(&[1, 1])));
    let timeline = tail_timeline(&movie);
    let required = timeline
        .continuation_dependencies(2_000, 3_000)
        .expect("fixture");
    assert!(required.contains(&ByteRange::new(100, 400)));
    assert!(required.iter().any(|span| span.start == 10_000));
    assert!(timeline.continuation_dependencies(2_000, 4_000).is_none());
}
