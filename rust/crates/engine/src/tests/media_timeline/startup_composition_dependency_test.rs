use crate::tests::media_timeline_dependency_support::{tail_timeline, with_sample_table};
use crate::tests::media_timeline_support::{full_box, values};

#[test]
fn startup_includes_decode_dependencies_of_reordered_presented_samples() {
    let mut ctts = full_box(
        b"ctts",
        values(&[4, 1, 2_000, 1, 0, 1, (-2_000_i32) as u32, 1, 0]),
    );
    ctts[8] = 1;
    let movie = with_sample_table(&[100, 200, 300, 400], ctts);
    let timeline = tail_timeline(&movie);
    let startup = timeline.startup_footprint().expect("decodable startup");

    assert!(
        startup
            .ranges()
            .iter()
            .any(|range| range.start <= 300 && range.end >= 400),
        "the frame presented at time zero is the third decoded sample"
    );
}
