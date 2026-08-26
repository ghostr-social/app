use super::super::decision_record_warp_test_support::{decision, record};
use super::reserve_support::{planned, record as reserve_record};
use crate::adaptive::{ActionKind, DecisionRecord, DecisionReplayStatus, PlannerCommand};
use crate::PostId;

#[test]
fn old_schema_two_reserve_shape_round_trips_without_hash_drift() {
    let decision = decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://origin.example/media".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    );
    let original = record(&decision);
    let json = serde_json::to_string(&original).expect("valid test fixture");
    let original_hash = original
        .replay_warp()
        .expect("valid test fixture")
        .integrity()
        .decision_hash()
        .to_owned();

    assert_legacy_shape(&json);
    let restored: DecisionRecord = serde_json::from_str(&json).expect("valid test fixture");
    assert_eq!(
        serde_json::to_string(&restored).expect("valid test fixture"),
        json
    );
    assert_eq!(
        restored
            .replay_warp()
            .expect("valid test fixture")
            .integrity()
            .decision_hash(),
        original_hash
    );
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
}

fn assert_legacy_shape(json: &str) {
    for absent in [
        "reserved_storage_bytes",
        "reserved_cpu_ms",
        "global_request_width",
        "authority_occupancy",
        "protected_action_ids",
        "chance",
        "degraded_reason",
    ] {
        assert!(!json.contains(absent), "legacy default emitted {absent}");
    }
}

#[test]
fn old_search_replay_reserve_defaults_reserialize_to_the_same_shape() {
    let (state, decision) = planned();
    let mut old =
        serde_json::to_value(reserve_record(&state, &decision)).expect("valid test fixture");
    let warp = old["warp_decision"]
        .as_object_mut()
        .expect("valid test fixture");
    strip_reserve(warp["reserve"].as_object_mut().expect("valid test fixture"));
    let replay = warp["search_replay_input"]
        .as_object_mut()
        .expect("valid test fixture");
    replay.remove("reserve");
    replay.remove("reserve_threshold_bps");
    replay.remove("reserve_degraded_reason");
    replay["budget"]
        .as_object_mut()
        .expect("valid test fixture")
        .remove("global_request_width");

    let restored: DecisionRecord = serde_json::from_value(old.clone()).expect("valid test fixture");
    assert_eq!(
        serde_json::to_value(restored).expect("valid test fixture"),
        old
    );
}

fn strip_reserve(value: &mut serde_json::Map<String, serde_json::Value>) {
    for field in [
        "reserved_storage_bytes",
        "reserved_cpu_ms",
        "global_request_width",
        "authority_occupancy",
        "protected_action_ids",
        "chance",
        "degraded_reason",
    ] {
        value.remove(field);
    }
}
