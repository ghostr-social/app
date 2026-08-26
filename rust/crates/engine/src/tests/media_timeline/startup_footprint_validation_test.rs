use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::cmaf_timeline_support::cmaf_sidx;
use crate::tests::media_timeline_support::{atom, classic_mdat_prefix, classic_moov, valid_ftyp};

#[test]
fn classic_startup_requires_a_valid_file_type_and_video_track() {
    let moov = classic_moov(&[100], &[100]);
    let invalid_ftyp = atom(b"ftyp", vec![0; 4]);
    let invalid_prefix = classic_mdat_prefix(&invalid_ftyp, 10_000, 200);
    let invalid = parse_mp4_segments(&[
        MediaSegment::new(0, &invalid_prefix),
        MediaSegment::new(10_000, &moov),
    ])
    .expect("valid test fixture");
    assert!(invalid.startup_footprint().is_none());

    let ftyp = valid_ftyp();
    let prefix = classic_mdat_prefix(&ftyp, 10_000, 200);
    let valid = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(10_000, &moov),
    ])
    .expect("valid test fixture");
    assert!(valid.startup_footprint().is_some());
}

#[test]
fn sidx_prediction_without_a_verified_fragment_is_not_startup() {
    let ftyp = valid_ftyp();
    let moov = atom(b"moov", atom(b"trak", Vec::new()));
    let sidx = cmaf_sidx(&[500], &[1_000], 0);
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &ftyp),
        MediaSegment::new(100, &moov),
        MediaSegment::new(1_000, &sidx),
    ])
    .expect("valid test fixture");

    assert!(timeline.startup_footprint().is_none());
}

#[test]
fn a_moov_shaped_sequence_inside_mdat_is_not_top_level_startup() {
    let mut bytes = valid_ftyp();
    bytes.extend(atom(b"mdat", classic_moov(&[32], &[4])));

    let result = parse_mp4_segments(&[MediaSegment::new(0, &bytes)]);

    assert!(result.is_err());
}
