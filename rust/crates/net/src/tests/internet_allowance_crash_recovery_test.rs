use crate::internet_allowance::{InternetAllowance, InternetDataLimit};

#[test]
fn a_crash_image_charges_outstanding_reservations_before_admitting_work() {
    let root = std::env::temp_dir().join(format!("warp-crashed-allowance-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("fixture");
    let path = root.join("ledger");
    let ledger = InternetAllowance::open(&path, InternetDataLimit::Bytes(100)).expect("fixture");
    let mut reservation = ledger.reserve(70).expect("fixture");
    reservation.started();
    reservation.received(10).expect("fixture");
    let crash_image = root.join("crash-image");
    std::fs::copy(&path, &crash_image).expect("fixture");
    let recovered =
        InternetAllowance::open(&crash_image, InternetDataLimit::Bytes(100)).expect("fixture");

    assert_eq!(recovered.usage().0, 70);
    assert_eq!(recovered.usage().1, 0);
    assert!(recovered.reserve(31).is_err());
    drop(recovered);
    drop(reservation);
    drop(ledger);
    std::fs::remove_dir_all(root).expect("fixture");
}
