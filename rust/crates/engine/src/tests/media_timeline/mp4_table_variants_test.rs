use crate::media_timeline::{parse_mp4_segments, MediaSegment, PlaybackWindow};
use crate::tests::media_timeline_support::{advanced_moov, atom, classic_moov, joined};
use crate::ByteRange;

#[test]
fn version_one_time_co64_fixed_sizes_and_multi_sample_chunks_are_supported() {
    let offset = 5_000_000_000_u64;
    let moov = advanced_moov(offset, 2, 250);
    let timeline = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).unwrap();

    assert_eq!(
        timeline.media_ranges(PlaybackWindow::try_new(0, 2_000).unwrap()),
        vec![ByteRange::new(offset, offset + 500)]
    );
}

#[test]
fn overlapping_tracks_produce_one_non_overlapping_range_intent() {
    let first = classic_moov(&[1_000], &[500]);
    let second = classic_moov(&[1_250], &[500]);
    let moov = atom(
        b"moov",
        joined(&[first[8..].to_vec(), second[8..].to_vec()]),
    );
    let timeline = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).unwrap();

    assert_eq!(
        timeline.media_ranges(PlaybackWindow::try_new(0, 1_000).unwrap()),
        vec![ByteRange::new(1_000, 1_750)]
    );
}
