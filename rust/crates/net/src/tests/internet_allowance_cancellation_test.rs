use crate::internet_allowance::{InternetAllowance, InternetDataLimit};

#[test]
fn abandoned_network_io_keeps_its_unproven_tail_charged() {
    let ledger = InternetAllowance::memory(InternetDataLimit::Bytes(100));
    let mut transfer = ledger.reserve(100).expect("fixture");
    transfer.started();
    transfer.received(10).expect("fixture");
    drop(transfer);

    assert_eq!(ledger.usage().0, 100);
    assert_eq!(ledger.usage().1, 0);
    assert!(ledger.reserve(1).is_err());
}
