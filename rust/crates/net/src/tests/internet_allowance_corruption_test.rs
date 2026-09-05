use crate::internet_allowance::{InternetAllowance, InternetDataLimit};

#[test]
fn corrupt_accounting_fails_closed_instead_of_resetting_usage() {
    let root = std::env::temp_dir().join(format!("warp-corrupt-allowance-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("fixture");
    let path = root.join("ledger");
    std::fs::write(&path, b"truncated accounting").expect("fixture");

    assert!(InternetAllowance::open(&path, InternetDataLimit::Bytes(100)).is_err());
    assert_eq!(
        std::fs::read(&path).expect("fixture"),
        b"truncated accounting"
    );
    std::fs::remove_dir_all(root).expect("fixture");
}
