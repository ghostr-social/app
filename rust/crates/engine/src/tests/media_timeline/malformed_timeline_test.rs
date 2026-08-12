use crate::media_timeline::{parse_mp4_segments, MediaSegment, TimelineError};
use crate::tests::cmaf_timeline_support::cmaf_sidx;
use crate::tests::media_timeline_support::{atom, classic_moov};

#[test]
fn a_declared_box_cut_short_is_reported_as_truncated() {
    let mut moov = classic_moov(&[1_000], &[100]);
    moov.truncate(moov.len() - 3);

    let result = parse_mp4_segments(&[MediaSegment::new(0, &moov)]);

    assert_eq!(result, Err(TimelineError::Truncated));
}

#[test]
fn an_invalid_media_timescale_is_rejected() {
    let mut moov = classic_moov(&[1_000], &[100]);
    let marker = moov.windows(4).position(|bytes| bytes == b"mdhd").unwrap();
    let timescale = marker + 16;
    moov[timescale..timescale + 4].fill(0);

    let result = parse_mp4_segments(&[MediaSegment::new(0, &moov)]);

    assert_eq!(result, Err(TimelineError::Malformed));
}

#[test]
fn missing_metadata_is_explicit() {
    assert_eq!(
        parse_mp4_segments(&[MediaSegment::new(0, b"not an mp4")]),
        Err(TimelineError::Unavailable)
    );
}

#[test]
fn absolute_box_offsets_cannot_wrap() {
    let moov = classic_moov(&[1_000], &[100]);

    let result = parse_mp4_segments(&[MediaSegment::new(u64::MAX - 4, &moov)]);

    assert_eq!(result, Err(TimelineError::Malformed));
}

#[test]
fn cmaf_first_offset_cannot_wrap() {
    let sidx = cmaf_sidx(&[100], &[1_000], u32::MAX);
    let start = u64::MAX - sidx.len() as u64 - u64::from(u32::MAX) + 1;

    let result = parse_mp4_segments(&[MediaSegment::new(start, &sidx)]);

    assert_eq!(result, Err(TimelineError::Malformed));
}

#[test]
fn malformed_nested_box_headers_are_rejected_without_panicking() {
    let declared_too_large = atom(b"moov", [100_u32.to_be_bytes(), *b"trak"].concat());
    let short_header = atom(b"moov", vec![0; 7]);
    let incomplete_extended = atom(
        b"moov",
        [1_u32.to_be_bytes(), *b"trak", 16_u32.to_be_bytes()].concat(),
    );

    assert_eq!(parse(&declared_too_large), Err(TimelineError::Truncated));
    assert_eq!(parse(&short_header), Err(TimelineError::Truncated));
    assert_eq!(parse(&incomplete_extended), Err(TimelineError::Truncated));
}

#[test]
fn timeline_extents_must_fit_the_known_representation_size() {
    let moov = classic_moov(&[100], &[100]);
    let start = 1_000;
    let timeline = parse_mp4_segments(&[MediaSegment::new(start, &moov)]).unwrap();
    let total = start + moov.len() as u64;

    assert!(timeline.fits_within(total));
    assert!(!timeline.fits_within(total - 1));
}

fn parse(bytes: &[u8]) -> Result<crate::media_timeline::MediaTimeline, TimelineError> {
    parse_mp4_segments(&[MediaSegment::new(0, bytes)])
}
