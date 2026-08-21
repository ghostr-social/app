use super::support::{bind, transfer_record};

#[test]
fn executed_request_serialization_never_contains_raw_post_or_source() {
    let mut record = transfer_record();
    assert!(bind(&mut record));

    let json = serde_json::to_string(&record).unwrap();
    for secret in ["secret-post", "origin.example", "token=raw"] {
        assert!(
            !json.contains(secret),
            "executed request leaked {secret}: {json}"
        );
    }
}
