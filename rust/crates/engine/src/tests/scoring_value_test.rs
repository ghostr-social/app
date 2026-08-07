use crate::scoring::value_per_byte;

#[test]
fn value_is_milestone_seconds_per_chunk_byte() {
    assert_eq!(value_per_byte(4.0, 1_000_000), 4.0e-6);
}

#[test]
fn more_seconds_for_the_same_bytes_is_worth_more() {
    assert!(value_per_byte(4.0, 1_000_000) > value_per_byte(2.0, 1_000_000));
}

#[test]
fn fewer_bytes_for_the_same_seconds_is_worth_more() {
    assert!(value_per_byte(4.0, 500_000) > value_per_byte(4.0, 1_000_000));
}

#[test]
fn a_zero_byte_chunk_does_not_divide_by_zero() {
    let value = value_per_byte(1.0, 0);

    assert!(value.is_finite());
    assert_eq!(value, 1.0);
}
