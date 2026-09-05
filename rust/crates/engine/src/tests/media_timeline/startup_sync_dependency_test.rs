use crate::tests::media_timeline_dependency_support::{tail_timeline, with_sample_table};
use crate::tests::media_timeline_support::{full_box, values};

#[test]
fn a_non_random_access_first_sample_cannot_certify_startup_at_zero() {
    let movie = with_sample_table(&[100, 200, 300], full_box(b"stss", values(&[1, 2])));
    let timeline = tail_timeline(&movie);

    assert!(
        timeline.startup_footprint().is_none(),
        "a later keyframe does not make the requested initial presentation decodable"
    );
}
