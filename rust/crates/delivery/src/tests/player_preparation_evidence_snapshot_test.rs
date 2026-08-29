use crate::delivery_events::{command_channel, PlayerPreparationClaim};
use ghostr_engine::{adaptive::AllocationPlan, PostId};

const POST: &str = "private-player-post";
const REPRESENTATION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ASSET: &str = "private-player-asset-token";

#[test]
fn player_preparation_evidence_exposes_only_pseudonymized_posts() {
    let (handle, mut commands) = command_channel();
    let claim = PlayerPreparationClaim::try_new(PostId::new(POST), REPRESENTATION, ASSET)
        .expect("valid player preparation claim");
    commands.publish_focused_plan_with_player_preparations(
        1,
        Some(PostId::new(POST)),
        AllocationPlan::default(),
        (vec![claim], Vec::new()),
    );

    let encoded = handle.evidence_page_json(0, 1).expect("evidence JSON");
    let snapshot: serde_json::Value = serde_json::from_str(&encoded).expect("valid JSON");
    let record = &snapshot["plan_page"]["records"][0];
    let verified = record["player_verified_post_ids"]
        .as_array()
        .expect("player verification evidence");

    assert_eq!(verified.len(), 1);
    assert_eq!(verified[0], record["current_post_id"]);
    assert!(!encoded.contains(POST));
    assert!(!encoded.contains(REPRESENTATION));
    assert!(!encoded.contains(ASSET));
}
