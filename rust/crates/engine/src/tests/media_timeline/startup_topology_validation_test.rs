use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_support::{atom, classic_mdat_prefix, classic_moov, valid_ftyp};

const MOVIE_START: u64 = 512;

#[test]
fn classic_startup_requires_a_proven_mdat_and_tail_box_boundary() {
    let movie = classic_moov(&[32], &[4]);
    let valid = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix_through_sample()),
        MediaSegment::new(MOVIE_START, &movie),
    ])
    .unwrap();
    let unproven = parse_mp4_segments(&[
        MediaSegment::new(0, &valid_ftyp()),
        MediaSegment::new(MOVIE_START, &movie),
    ])
    .unwrap();

    assert!(valid.startup_footprint().is_some());
    assert!(unproven.startup_footprint().is_none());
}

#[test]
fn a_moov_inside_a_nonzero_mdat_segment_is_not_startup() {
    let nested = atom(b"mdat", classic_moov(&[32], &[4]));
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &valid_ftyp()),
        MediaSegment::new(MOVIE_START, &nested),
    ])
    .unwrap();

    assert!(timeline.startup_footprint().is_none());
}

#[test]
fn a_nested_tail_moov_cannot_borrow_a_proven_head_mdat() {
    let movie = classic_moov(&[32], &[4]);
    let prefix = classic_mdat_prefix(&valid_ftyp(), 1_024, 36);
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(MOVIE_START, &movie),
    ])
    .unwrap();

    assert!(timeline.startup_footprint().is_none());
}

#[test]
fn a_nested_movie_cannot_shadow_a_later_top_level_movie() {
    let nested = classic_moov(&[32], &[4]);
    let top_level = classic_moov(&[32], &[4]);
    let prefix = classic_mdat_prefix(&valid_ftyp(), MOVIE_START as u32, 36);
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(40, &nested),
        MediaSegment::new(MOVIE_START, &top_level),
    ])
    .unwrap();

    assert!(timeline.startup_footprint().is_some());
}

#[test]
fn a_size_zero_file_type_cannot_authorize_sparse_tail_metadata() {
    let mut open_ended = 0_u32.to_be_bytes().to_vec();
    open_ended.extend(b"ftyp");
    open_ended.extend(b"isom\0\0\0\0");
    let movie = classic_moov(&[32], &[4]);
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &open_ended),
        MediaSegment::new(MOVIE_START, &movie),
    ])
    .unwrap();

    assert!(timeline.startup_footprint().is_none());
}

#[test]
fn missing_metadata_payload_breaks_later_top_level_authority() {
    let prefix = classic_mdat_prefix(&valid_ftyp(), 64, 64);
    let movie = classic_moov(&[32], &[4]);
    let mut missing_movie = (MOVIE_START as u32 - 64).to_be_bytes().to_vec();
    missing_movie.extend(b"moov");
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(64, &missing_movie),
        MediaSegment::new(MOVIE_START, &movie),
    ])
    .unwrap();

    assert!(timeline.startup_footprint().is_none());
}

fn prefix_through_sample() -> Vec<u8> {
    classic_mdat_prefix(&valid_ftyp(), MOVIE_START as u32, 36)
}
