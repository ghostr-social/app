#[test]
fn rejected_admission_never_reports_an_accepted_promotion() {
    let result = super::rejected_result(Some(16), Some(false), true);

    assert!(result.cancelled);
    assert!(!result.promoted);
    assert_eq!(result.bytes_written, 0);
}
