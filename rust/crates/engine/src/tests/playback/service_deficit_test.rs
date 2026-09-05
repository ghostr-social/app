use crate::playback::{BufferScenario, PlaybackPhase, UsableArrival};

#[test]
fn burst_arrivals_use_the_left_limit_not_the_mean_service_rate() {
    let scenario = BufferScenario::new(20_000, 1_000, PlaybackPhase::Playing);
    let arrivals: Vec<_> = (1..=20)
        .map(|second| UsableArrival::new(second * 1_000, second * 800))
        .collect();
    assert_eq!(scenario.required_ms(&arrivals), Ok(4_800));
    assert_eq!(scenario.continuous_required_ms(800), 4_000);
}

#[test]
fn tied_arrivals_extend_one_contiguous_frontier_without_double_counting() {
    let scenario = BufferScenario::new(2_000, 1_000, PlaybackPhase::Playing);
    let arrivals = [
        UsableArrival::new(1_000, 500),
        UsableArrival::new(1_000, 800),
    ];
    assert_eq!(scenario.required_ms(&arrivals), Ok(1_200));
    let duplicate = [
        UsableArrival::new(1_000, 800),
        UsableArrival::new(1_000, 800),
    ];
    assert_eq!(scenario.required_ms(&duplicate), Ok(1_200));
}

#[test]
fn exact_deadline_arrival_and_eof_do_not_create_extra_consumption() {
    let scenario = BufferScenario::new(1_000, 1_000, PlaybackPhase::Playing);
    assert_eq!(
        scenario.required_ms(&[UsableArrival::new(1_000, 5_000)]),
        Ok(1_000)
    );
    assert_eq!(
        scenario.required_ms(&[UsableArrival::new(2_000, 5_000)]),
        Ok(1_000)
    );
}
