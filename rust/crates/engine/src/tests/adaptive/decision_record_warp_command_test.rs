use super::decision_record_warp_test_support::{allocation, decision, record};
use crate::adaptive::{ActionKind, PlannerCommand, RetrievalRequest, TransformKind};
use crate::{ActionId, ByteRange, PostId};

#[test]
fn every_planner_command_has_a_typed_authoritative_record() {
    let source = "https://origin.example/media";
    let transfer = || {
        allocation(
            source,
            RetrievalRequest::FetchRange {
                bytes: ByteRange::new(0, 64),
                promotion: None,
            },
        )
    };
    let cases = [
        (
            PlannerCommand::ProbeHead {
                post: PostId::new("secret-post"),
                source: source.into(),
            },
            ActionKind::Head,
            "probe_head",
        ),
        (
            PlannerCommand::Transfer(transfer()),
            ActionKind::FetchRange(ByteRange::new(0, 64)),
            "transfer",
        ),
        (
            PlannerCommand::Promote {
                post: PostId::new("secret-post"),
                action: ActionId::new(2),
            },
            ActionKind::Promote {
                active: ActionId::new(2),
                maximum_bytes: 128,
            },
            "promote",
        ),
        (
            PlannerCommand::Transform {
                post: PostId::new("secret-post"),
                kind: TransformKind::Remux,
            },
            ActionKind::Transform(TransformKind::Remux),
            "transform",
        ),
        (
            PlannerCommand::Hedge {
                primary: ActionId::new(3),
                transfer: transfer(),
            },
            ActionKind::Hedge {
                primary: ActionId::new(3),
                alternate: source.into(),
            },
            "hedge",
        ),
        (
            PlannerCommand::Cancel(ActionId::new(4)),
            ActionKind::Cancel(ActionId::new(4)),
            "cancel",
        ),
    ];

    for (command, kind, command_tag) in cases {
        let value = serde_json::to_value(record(&decision("secret-post", command, kind))).unwrap();
        let selected = &value["warp_decision"]["selected"];
        assert_eq!(selected["command"]["command"], command_tag);
        assert!(!value["chosen_action"].is_null());
    }
}
