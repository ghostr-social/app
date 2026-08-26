use crate::media_timeline::{
    parse_mp4_segments, parse_mp4_segments_with_control, MediaSegment, TimelineError,
    TimelineParseControl,
};
use crate::tests::media_timeline_support::{advanced_moov, classic_moov};
use core::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn byte_scanning_stops_at_a_deterministic_cancellation_checkpoint() {
    let control = CancelAfter::new(50);
    let input = vec![0_u8; 8 * 1024 * 1024];

    let result = parse_mp4_segments_with_control(&[MediaSegment::new(0, &input)], &control);

    assert_eq!(result, Err(TimelineError::Cancelled));
    assert_eq!(control.polls(), 50);
}

#[test]
fn a_large_sample_table_pipeline_stops_at_a_deterministic_checkpoint() {
    let control = CancelAfter::new(450);
    let moov = advanced_moov(1_000, 100_000, 1);

    let result = parse_mp4_segments_with_control(&[MediaSegment::new(0, &moov)], &control);

    assert_eq!(result, Err(TimelineError::Cancelled));
    assert_eq!(control.polls(), 450);
}

#[test]
fn the_existing_parser_is_the_never_cancelled_wrapper() {
    let moov = classic_moov(&[100], &[10]);
    let segment = [MediaSegment::new(0, &moov)];

    assert_eq!(
        parse_mp4_segments(&segment),
        parse_mp4_segments_with_control(&segment, &NeverCancelled)
    );
}

struct CancelAfter {
    polls: AtomicUsize,
    cancel_at: usize,
}

impl CancelAfter {
    fn new(cancel_at: usize) -> Self {
        Self {
            polls: AtomicUsize::new(0),
            cancel_at,
        }
    }

    fn polls(&self) -> usize {
        self.polls.load(Ordering::Acquire)
    }
}

impl TimelineParseControl for CancelAfter {
    fn is_cancelled(&self) -> bool {
        self.polls.fetch_add(1, Ordering::AcqRel) + 1 >= self.cancel_at
    }
}

struct NeverCancelled;

impl TimelineParseControl for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}
