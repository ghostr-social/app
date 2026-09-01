use serde_json::Value;

type Summary = (u64, Option<u64>, String, u64, u64, bool);

pub(super) fn summarize(history: &str) -> Vec<Summary> {
    let Ok(value) = serde_json::from_str::<Value>(history) else {
        return Vec::new();
    };
    value["decisions"]["records"]
        .as_array()
        .into_iter()
        .flatten()
        .rev()
        .take(12)
        .map(summary)
        .collect()
}

fn summary(record: &Value) -> Summary {
    let action = &record["chosen_action"];
    (
        record["sequence"].as_u64().unwrap_or_default(),
        record["chosen_action_id"].as_u64(),
        outcome(record),
        action["bytes_start"].as_u64().unwrap_or_default(),
        action["bytes_end"].as_u64().unwrap_or_default(),
        record["warp_decision"]["additional_request_slot_demanded"]
            .as_bool()
            .unwrap_or(false),
    )
}

fn outcome(record: &Value) -> String {
    record["eventual_outcome"]["status"]
        .as_str()
        .unwrap_or("unknown")
        .to_owned()
}
