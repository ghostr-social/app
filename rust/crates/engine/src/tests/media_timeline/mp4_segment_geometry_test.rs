use crate::media_timeline::{parse_mp4_segments, MediaSegment, TimelineError};
use crate::tests::cmaf_timeline_support::cmaf_sidx;
use crate::tests::media_timeline_support::classic_moov;

#[test]
fn duplicate_absolute_segments_are_rejected_before_timeline_fusion() {
    let moov = classic_moov(&[1_000], &[1_000]);
    assert!(parse_mp4_segments(&[MediaSegment::new(0, &moov)]).is_ok());
    let segments = [MediaSegment::new(0, &moov), MediaSegment::new(0, &moov)];

    assert_eq!(parse_mp4_segments(&segments), Err(TimelineError::Malformed));
}

#[test]
fn partial_absolute_overlap_is_rejected_even_when_one_segment_is_valid() {
    let moov = classic_moov(&[1_000], &[1_000]);
    assert!(parse_mp4_segments(&[MediaSegment::new(0, &moov)]).is_ok());
    let segments = [
        MediaSegment::new(0, &moov),
        MediaSegment::new(4, &moov[4..]),
    ];

    assert_eq!(parse_mp4_segments(&segments), Err(TimelineError::Malformed));
}

#[test]
fn disjoint_segment_order_and_empty_spans_do_not_change_the_timeline() {
    let first = cmaf_sidx(&[100], &[1_000], 100);
    let second = cmaf_sidx(&[200], &[1_000], 200);
    let second_start = first.len() as u64;
    let sorted = [
        MediaSegment::new(0, &first),
        MediaSegment::new(second_start, &second),
    ];
    let unordered = [
        MediaSegment::new(second_start, &second),
        MediaSegment::new(u64::MAX, &[]),
        MediaSegment::new(0, &first),
    ];

    assert_eq!(parse_mp4_segments(&unordered), parse_mp4_segments(&sorted));
}
