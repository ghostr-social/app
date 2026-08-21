use super::support::{bind, transfer_record};
use crate::adaptive::{DecisionRecord, DecisionReplayStatus};

#[test]
fn replay_rejects_every_executed_request_contract_mutation() {
    let mut record = transfer_record();
    assert!(bind(&mut record));
    let original = serde_json::to_value(record).unwrap();
    let mutations = [
        ("post_id", serde_json::json!("changed")),
        ("source_id", serde_json::json!("https://changed.invalid/x")),
        ("request.bytes_end", serde_json::json!(129)),
        ("resources.network_bytes", serde_json::json!(31)),
    ];

    for (path, replacement) in mutations {
        let mut value = original.clone();
        replace(&mut value["executed_request"], path, replacement);
        let tampered: DecisionRecord = serde_json::from_value(value).unwrap();
        assert_eq!(
            tampered.replay(),
            DecisionReplayStatus::PlanMismatch,
            "{path}"
        );
    }
}

fn replace(value: &mut serde_json::Value, path: &str, replacement: serde_json::Value) {
    let mut target = value;
    let mut fields = path.split('.').peekable();
    while let Some(field) = fields.next() {
        if fields.peek().is_none() {
            target[field] = replacement;
            return;
        }
        target = &mut target[field];
    }
}
