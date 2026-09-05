use crate::internet_allowance::{InternetAllowance, InternetDataLimit};

#[test]
fn completed_usage_survives_restart() {
    let directory = std::env::temp_dir().join(format!("warp-allowance-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("fixture");
    let path = directory.join("ledger");
    let ledger = InternetAllowance::open(&path, InternetDataLimit::Bytes(100)).expect("fixture");
    let mut transfer = ledger.reserve(70).expect("fixture");
    transfer.started();
    transfer.received(40).expect("fixture");
    transfer.complete().expect("fixture");
    drop(transfer);
    drop(ledger);
    let reopened = InternetAllowance::open(&path, InternetDataLimit::Bytes(100)).expect("fixture");

    assert_eq!(reopened.usage().0, 40);
    assert!(reopened.reserve(61).is_err());
    drop(reopened);
    std::fs::remove_dir_all(directory).expect("fixture");
}
