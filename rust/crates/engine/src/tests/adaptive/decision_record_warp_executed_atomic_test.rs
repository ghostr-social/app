use super::support::{executed, privacy, transfer_record, SOURCE};
use crate::adaptive::{
    ActionKind, ExecutedRequest, PlannerCommand, ResourceCost, RetrievalRequest, TransformKind,
};
use crate::tests::adaptive::decision_record_warp_test_support::{decision, record};
use crate::{ActionId, ByteRange, PostId};

#[test]
fn invalid_request_bindings_leave_action_and_observation_unbound() {
    let invalid = [
        ExecutedRequest {
            post: PostId::new("other-post"),
            ..executed()
        },
        ExecutedRequest {
            source: "https://other.example/media".into(),
            ..executed()
        },
        ExecutedRequest {
            request: range(96, 160),
            resources: ResourceCost::new(64, 64, 0, 1),
            ..executed()
        },
        ExecutedRequest {
            resources: ResourceCost::new(31, 32, 0, 1),
            ..executed()
        },
    ];
    for request in invalid {
        let mut record = transfer_record();
        assert!(!record.bind_executed_action(ActionId::new(44), &request, &privacy()));
        assert_eq!(record.chosen_action_id, None);
        assert_eq!(record.executed_request, None);
    }
}

#[test]
fn ordinary_binding_is_reserved_for_non_request_actions() {
    let mut request = transfer_record();
    assert!(!request.bind_action(ActionId::new(44)));

    let mut transform = record(&decision(
        "secret-post",
        PlannerCommand::Transform {
            post: PostId::new("secret-post"),
            kind: TransformKind::Remux,
        },
        ActionKind::Transform(TransformKind::Remux),
    ));
    assert!(transform.bind_action(ActionId::new(45)));
    assert_eq!(transform.executed_request, None);
    assert!(SOURCE.starts_with("https://"));
}

fn range(start: u64, end: u64) -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(start, end),
        promotion: None,
    }
}
