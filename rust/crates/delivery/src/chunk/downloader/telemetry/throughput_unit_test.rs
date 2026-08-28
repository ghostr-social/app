use super::throughput;
use core::time::Duration;

#[test]
fn origin_throughput_is_recorded_in_bits_per_second_after_ttfb() {
    assert_eq!(
        throughput(
            1_000,
            Duration::from_millis(1_100),
            Some(Duration::from_millis(100)),
        ),
        Some(8_000),
    );
}
