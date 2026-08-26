use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::cmaf_timeline_support::cmaf_sidx;
use crate::tests::media_timeline_assertions::{duration_ms, media_ranges};
use crate::ByteRange;

#[test]
fn cmaf_sidx_references_map_time_to_non_overlapping_segments() {
    let sidx = cmaf_sidx(&[500, 1_200, 300], &[1_000, 2_000, 1_000], 20);
    let absolute_start = 128_u64;
    let media_start = absolute_start + sidx.len() as u64 + 20;
    let timeline = parse_mp4_segments(&[MediaSegment::new(absolute_start, &sidx)])
        .expect("valid test fixture");

    let initial = media_ranges(&timeline, 0, 1_000);
    let middle = media_ranges(&timeline, 1_000, 3_000);

    assert_eq!(
        initial,
        vec![ByteRange::new(media_start, media_start + 500)]
    );
    assert_eq!(
        middle,
        vec![ByteRange::new(media_start + 500, media_start + 1_700)]
    );
    assert_eq!(duration_ms(&timeline), 4_000);
}
