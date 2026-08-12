use crate::adaptive::{SnapshotError, ViewProbability};

#[test]
fn view_probability_rejects_non_finite_and_out_of_range_values() {
    for invalid in [f64::NAN, -0.01, 1.01] {
        assert_eq!(
            ViewProbability::new(invalid),
            Err(SnapshotError::InvalidProbability)
        );
    }
}
