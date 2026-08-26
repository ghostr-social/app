use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::media_timeline_assertions::{duration_ms, media_ranges};
use crate::tests::media_timeline_support::classic_moov;
use crate::ByteRange;

#[test]
fn vbr_samples_map_playback_time_to_their_exact_byte_extents() {
    let moov = classic_moov(&[1_000, 1_500, 3_000, 3_500], &[100, 900, 200, 800]);
    let timeline =
        parse_mp4_segments(&[MediaSegment::new(400, &moov)]).expect("valid test fixture");

    let first = media_ranges(&timeline, 0, 1_000);
    let second = media_ranges(&timeline, 1_000, 2_000);
    let first_two = media_ranges(&timeline, 0, 2_000);

    assert_eq!(first, vec![ByteRange::new(1_000, 1_100)]);
    assert_eq!(second, vec![ByteRange::new(1_500, 2_400)]);
    assert_eq!(
        first_two,
        vec![ByteRange::new(1_000, 1_100), ByteRange::new(1_500, 2_400)]
    );
    assert_eq!(duration_ms(&timeline), 4_000);
}
