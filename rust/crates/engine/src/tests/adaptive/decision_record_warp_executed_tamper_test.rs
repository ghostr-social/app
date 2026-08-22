use super::support::{bind, transfer_record};
use crate::adaptive::{DecisionOutcome, DecisionRecord, DecisionReplayStatus, ResourceCost};

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

#[test]
fn terminal_seal_binds_a_coherent_executed_range() {
    let mut record = transfer_record();
    assert!(bind(&mut record));
    assert!(record.resolve_with_resources(
        DecisionOutcome::Succeeded {
            bytes: 32,
            elapsed_ms: 4,
        },
        ResourceCost::new(32, 32, 0, 1),
    ));
    let mut value = serde_json::to_value(record).unwrap();
    replace(
        &mut value["executed_request"],
        "request.bytes_start",
        serde_json::json!(17),
    );
    replace(
        &mut value["executed_request"],
        "request.bytes_end",
        serde_json::json!(49),
    );

    let tampered: DecisionRecord = serde_json::from_value(value).unwrap();
    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
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
