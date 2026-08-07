#![cfg(unix)]

use crate::partial_range_store::free_space::checked_available_bytes;

#[test]
fn free_space_operands_are_widened_before_multiplication() {
    let available = checked_available_bytes(u32::MAX, 4096_u32);

    assert_eq!(available, Some(u64::from(u32::MAX) * 4096));
}
