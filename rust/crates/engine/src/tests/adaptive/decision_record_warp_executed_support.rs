use crate::adaptive::{
    ActionKind, DecisionPrivacy, DecisionRecord, ExecutedRequest, PlannerCommand, ResourceCost,
    RetrievalRequest,
};
use crate::tests::adaptive::decision_record_warp_test_support::{allocation, decision, record};
use crate::{ActionId, ByteRange, PostId};

pub(super) const SOURCE: &str = "https://origin.example/private.mp4?token=raw";

pub(super) fn transfer_record() -> DecisionRecord {
    record(&decision(
        "secret-post",
        PlannerCommand::Transfer(allocation(SOURCE, intent())),
        ActionKind::FetchRange(ByteRange::new(0, 128)),
    ))
}

pub(super) fn hedge_record() -> DecisionRecord {
    record(&decision(
        "secret-post",
        PlannerCommand::Hedge {
            primary: ActionId::new(5),
            transfer: allocation(SOURCE, intent()),
        },
        ActionKind::Hedge {
            primary: ActionId::new(5),
            alternate: SOURCE.into(),
        },
    ))
}

pub(super) fn executed() -> ExecutedRequest {
    ExecutedRequest {
        post: PostId::new("secret-post"),
        source: SOURCE.into(),
        request: RetrievalRequest::FetchRange {
            bytes: ByteRange::new(16, 48),
            promotion: None,
        },
        resources: ResourceCost::new(32, 32, 0, 1),
    }
}

pub(super) fn bind(record: &mut DecisionRecord) -> bool {
    record.bind_executed_action(ActionId::new(44), &executed(), &privacy())
}

pub(super) fn privacy() -> DecisionPrivacy {
    DecisionPrivacy::from_key([5; 32])
}

fn intent() -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 128),
        promotion: None,
    }
}
