use crate::delivery_events::command_channel;
use crate::tests::decision_log_fixture::{selected_warp, work};
use ghostr_engine::adaptive::{
    Allocation, AllocationPlan, AllocationReason, CandidateUtility, PreemptionAuthority,
    RetrievalRequest,
};
use ghostr_engine::{ByteRange, PostId};

const SOURCE: &str = "https://private.example/account/video.mp4?token=secret";

#[test]
fn delivery_evidence_json_is_bounded_versioned_and_privacy_safe() {
    let (handle, mut commands) = command_channel();
    for observed_at_ms in 1..=66 {
        commands.publish_focused_plan(observed_at_ms, Some(PostId::new("event-id")), plan());
    }

    let encoded = handle
        .evidence_page_json(0, usize::MAX)
        .expect("evidence JSON");
    let snapshot: serde_json::Value = serde_json::from_str(&encoded).expect("valid JSON");
    assert_eq!(snapshot["schema_version"], 1);
    assert_eq!(
        snapshot["plan_page"]["records"].as_array().expect("valid test fixture").len(),
        64
    );
    assert_eq!(snapshot["plan_page"]["records"][0]["revision"], 1);
    assert_eq!(snapshot["plan_page"]["oldest_retained_revision"], 1);
    assert_eq!(snapshot["plan_page"]["latest_retained_revision"], 66);
    assert_eq!(snapshot["plan_page"]["cursor_truncated"], false);
    assert_eq!(snapshot["plan_page"]["has_more"], true);
    assert_ne!(
        snapshot["plan_page"]["records"][0]["current_post_id"],
        "event-id",
    );
    assert!(!encoded.contains(SOURCE));
    assert!(snapshot["evaluation"]["readiness"].is_object());

    let tail = handle.evidence_page_json(64, usize::MAX).expect("valid test fixture");
    let tail: serde_json::Value = serde_json::from_str(&tail).expect("valid test fixture");
    assert_eq!(tail["plan_page"]["records"].as_array().expect("valid test fixture").len(), 2);
    assert_eq!(tail["plan_page"]["records"][0]["revision"], 65);
    assert_eq!(tail["plan_page"]["has_more"], false);

    let work = work();
    let (sequence, _token) = selected_warp(&handle, &commands, &work);
    let decisions = handle.decision_history_json().expect("decision JSON");
    let decisions: serde_json::Value = serde_json::from_str(&decisions).expect("valid test fixture");
    assert_eq!(decisions["schema_version"], 1);
    assert!(decisions["decisions"]["records"].is_array());
    assert_eq!(decisions["integrity"][0]["sequence"], sequence);
    assert_eq!(decisions["integrity"][0]["status"], "verified");
    assert_eq!(decisions["integrity"][0]["search_status"], "verified");
}

fn plan() -> AllocationPlan {
    AllocationPlan {
        allocations: vec![Allocation {
            post: PostId::new("event-id"),
            request: RetrievalRequest::FetchRange {
                bytes: ByteRange::new(65_536, 131_072),
                promotion: None,
            },
            source: SOURCE.into(),
            expected_playable_gain_ms: 500,
            utility: CandidateUtility {
                view_probability: 0.5,
                additional_playable_ms: 500,
                expected_delivery_ms: 100,
                score: 1.0,
            },
            authority: PreemptionAuthority::Transition,
            commitment_until_ms: 200,
            reason: AllocationReason::NextStartability,
        }],
        ..AllocationPlan::default()
    }
}
