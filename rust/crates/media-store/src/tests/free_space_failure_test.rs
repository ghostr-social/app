#[cfg(unix)]
use crate::partial_range_store::free_space::statvfs_available;

#[cfg(unix)]
#[test]
fn failed_statvfs_reports_unknown_space_without_reading_output() {
    let available = statvfs_available(-1, || -> Option<u64> {
        panic!("failed statvfs output must stay unread")
    });

    assert_eq!(available, None);
}
