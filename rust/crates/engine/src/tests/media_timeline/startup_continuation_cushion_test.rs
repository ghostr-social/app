use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_support::{classic_mdat_prefix, classic_moov, valid_ftyp};
use crate::ByteRange;

#[test]
fn startup_requires_a_bounded_continuation_after_the_first_sample() {
    let prefix = classic_mdat_prefix(&valid_ftyp(), 1_024, 132);
    let movie = classic_moov(&[32, 200, 400], &[100, 100, 100]);
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(1_024, &movie),
    ])
    .expect("valid MP4 sample tables");
    let startup = timeline
        .startup_footprint()
        .expect("supported native startup");

    assert_eq!(
        startup.playable_ms(),
        2_000,
        "one frame is not a continuation cushion"
    );
    assert!(
        startup.ranges().contains(&ByteRange::new(200, 300)),
        "startup needs the second second"
    );
    assert!(
        !startup.ranges().contains(&ByteRange::new(400, 500)),
        "startup must remain bounded"
    );
}
