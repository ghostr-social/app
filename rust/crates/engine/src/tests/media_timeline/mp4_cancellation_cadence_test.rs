use crate::media_timeline::{
    parse_mp4_segments_with_control, MediaSegment, TimelineError, TimelineParseControl,
};
use std::sync::atomic::{AtomicUsize, Ordering};

const INPUT_BYTES: usize = 8 * 1024 * 1024;
const MAXIMUM_BYTES_BETWEEN_POLLS: usize = 4 * 1024;

#[test]
fn a_full_junk_scan_polls_at_least_once_per_bounded_byte_block() {
    let control = CountingControl::default();
    let input = vec![0_u8; INPUT_BYTES];

    let result = parse_mp4_segments_with_control(&[MediaSegment::new(0, &input)], &control);

    assert_eq!(result, Err(TimelineError::Unavailable));
    assert!(control.polls() >= INPUT_BYTES / MAXIMUM_BYTES_BETWEEN_POLLS);
}

#[derive(Default)]
struct CountingControl(AtomicUsize);

impl CountingControl {
    fn polls(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }
}

impl TimelineParseControl for CountingControl {
    fn is_cancelled(&self) -> bool {
        self.0.fetch_add(1, Ordering::AcqRel);
        false
    }
}
