use crate::manager::resource_control::cursor;

#[test]
fn producer_cursor_carries_revision_wrap_into_the_epoch() {
    let before = cursor(u128::from(u64::MAX));
    let after = cursor(u128::from(u64::MAX) + 1);

    assert_eq!((before.epoch, before.revision), (0, u64::MAX));
    assert_eq!((after.epoch, after.revision), (1, 0));
}
