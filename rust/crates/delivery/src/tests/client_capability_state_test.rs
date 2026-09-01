use crate::client_capability::axiom_test_support::ClientCapabilityState;
use crate::client_capability::{
    ClientCapabilityModel, ClientCapabilityProfile, ClientCapabilityStatus,
};
use serde_json::{json, Value};

#[test]
fn restored_capability_state_is_validated_and_bounded() {
    let records: Vec<_> = (0..140)
        .map(|index| record(&format!("representation-{index}"), &[index]))
        .chain([record("", &[])])
        .collect();
    let state: ClientCapabilityState = serde_json::from_value(json!({
        "generation": 9,
        "records": records,
    }))
    .expect("valid test fixture");

    let model = ClientCapabilityModel::from_state(state);
    let persisted = serde_json::to_value(model.state()).expect("valid test fixture");
    assert_eq!(
        persisted["records"]
            .as_array()
            .expect("valid test fixture")
            .len(),
        128
    );
    assert!(!persisted.to_string().contains("\"representation\":\"\""));
    assert_eq!(
        model.status(9, &profile("representation-139")),
        ClientCapabilityStatus::Supported {
            p95_first_frame_us: 139,
        },
    );
    let unknown = ClientCapabilityProfile::try_new("missing", Some("hvc1"), Some((1080, 1920)))
        .expect("valid test fixture");
    assert_eq!(model.status(9, &unknown), ClientCapabilityStatus::Unknown);
}

fn record(representation: &str, samples: &[u64]) -> Value {
    json!({
        "profile": {
            "representation": representation,
            "codec": "avc1",
            "dimensions": [1080, 1920],
            "persistent": true,
        },
        "result": {"Supported": {"first_frame_us": samples}},
    })
}

fn profile(representation: &str) -> ClientCapabilityProfile {
    ClientCapabilityProfile::try_new(representation, Some("avc1"), Some((1080, 1920)))
        .expect("valid test fixture")
        .with_persistent_identity(true)
}
