use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::cmaf_timeline_support::cmaf_sidx;
use crate::tests::media_timeline_assertions::media_ranges;
use crate::tests::media_timeline_support::{atom, joined};
use crate::ByteRange;

#[test]
fn fragmented_init_tracks_without_sample_tables_defer_to_sidx() {
    let empty_trak = atom(b"trak", Vec::new());
    let empty_mdia = atom(b"trak", atom(b"mdia", Vec::new()));
    let mdhd_only = atom(b"trak", atom(b"mdia", atom(b"mdhd", Vec::new())));
    let no_stbl = atom(
        b"trak",
        atom(
            b"mdia",
            joined(&[atom(b"mdhd", Vec::new()), atom(b"minf", Vec::new())]),
        ),
    );
    let moov = atom(
        b"moov",
        joined(&[empty_trak, empty_mdia, mdhd_only, no_stbl]),
    );
    let sidx = cmaf_sidx(&[500], &[1_000], 0);
    let media_start = 1_000 + sidx.len() as u64;

    let timeline =
        parse_mp4_segments(&[MediaSegment::new(0, &moov), MediaSegment::new(1_000, &sidx)])
            .expect("valid test fixture");

    assert_eq!(
        media_ranges(&timeline, 0, 1_000),
        vec![ByteRange::new(media_start, media_start + 500)]
    );
}
