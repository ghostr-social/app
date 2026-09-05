use crate::tests::media_timeline_dependency_support::tail_timeline;
use crate::tests::media_timeline_support::{atom, classic_moov, joined};

#[test]
fn startup_coverage_ends_at_the_shortest_required_track() {
    let video = classic_moov(&[100, 200, 300], &[100, 100, 100]);
    let mut audio = classic_moov(&[400], &[100]);
    let handler = audio
        .windows(4)
        .position(|bytes| bytes == b"vide")
        .expect("fixture");
    audio[handler..handler + 4].copy_from_slice(b"soun");
    let movie = atom(b"moov", joined(&[video[8..].to_vec(), audio[8..].to_vec()]));
    let timeline = tail_timeline(&movie);

    assert_eq!(
        timeline.startup_footprint().expect("fixture").playable_ms(),
        1_000
    );
}
