use crate::internet_allowance::{InternetAllowance, InternetDataLimit};

#[test]
fn concurrent_reservations_cannot_spend_the_same_cumulative_allowance() {
    let ledger = InternetAllowance::memory(InternetDataLimit::Bytes(100));
    let mut first = ledger.reserve(70).expect("fixture");
    assert!(ledger.reserve(31).is_err());
    first.started();
    first.received(40).expect("fixture");
    first.complete().expect("fixture");

    assert_eq!(ledger.usage().0, 40);
    assert_eq!(ledger.usage().1, 0);
    let second = ledger.reserve(60).expect("fixture");
    assert!(ledger.reserve(1).is_err());
    drop(second);
    assert_eq!(ledger.usage().0, 40);
    assert_eq!(ledger.usage().1, 0);
}
