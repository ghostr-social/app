use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_assertions::required_ranges;
use crate::tests::media_timeline_support::classic_moov;
use crate::ByteRange;

#[test]
fn a_tail_moov_authorizes_early_samples_without_the_middle_of_the_file() {
    let moov = classic_moov(&[32, 500], &[100, 600]);
    let moov_start = 10_000_u64;
    let segment_start = moov_start - 200;
    let mut tail = vec![0xa5; 200];
    tail.extend_from_slice(&moov);

    let timeline =
        parse_mp4_segments(&[MediaSegment::new(segment_start, &tail)]).expect("valid test fixture");
    let startup = required_ranges(&timeline, 0, 1_000);

    assert_eq!(
        startup,
        vec![
            ByteRange::new(32, 132),
            ByteRange::new(moov_start, moov_start + moov.len() as u64),
        ]
    );
    assert!(!startup
        .iter()
        .any(|range| range.start == 132 && range.end == moov_start));
}
