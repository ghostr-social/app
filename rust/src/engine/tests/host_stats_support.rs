pub fn assert_close(actual: f64, expected: f64) {
    let delta = (actual - expected).abs();
    assert!(delta < 1e-9, "expected {expected}, got {actual} (delta {delta})");
}
