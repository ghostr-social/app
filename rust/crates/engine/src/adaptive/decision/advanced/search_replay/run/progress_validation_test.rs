use super::valid_progress;
use crate::adaptive::warp::SearchReplayMode;
use crate::adaptive::{ActionKind, ActionNode, ActionValue};
use crate::PostId;

#[test]
fn recorded_progress_requires_unique_existing_least_risk_actions() {
    let nodes = vec![ActionNode::new(
        7,
        PostId::new("p1"),
        ActionKind::Cancel(crate::ActionId::new(1)),
        ActionValue::default(),
    )];

    assert!(valid_progress(SearchReplayMode::LeastRisk, &[7], &nodes));
    assert!(!valid_progress(SearchReplayMode::Beam, &[7], &nodes));
    assert!(!valid_progress(
        SearchReplayMode::LeastRisk,
        &[7, 7],
        &nodes,
    ));
    assert!(!valid_progress(SearchReplayMode::LeastRisk, &[8], &nodes,));
}
