use super::decision_record_warp_test_support::{decision, record};
use crate::adaptive::{ActionKind, PlannerCommand, TransformKind};
use crate::{ActionId, ByteRange, PostId};

#[test]
fn every_action_kind_has_a_typed_authoritative_record() {
    let range = ByteRange::new(4, 8);
    let cases = [
        (ActionKind::Head, "head", None),
        (ActionKind::Prefix(range), "prefix", None),
        (ActionKind::Tail(range), "tail", None),
        (ActionKind::FetchRange(range), "fetch_range", None),
        (
            ActionKind::FetchWhole { maximum_bytes: 8 },
            "fetch_whole",
            None,
        ),
        (
            ActionKind::Promote {
                active: ActionId::new(1),
                maximum_bytes: 8,
            },
            "promote",
            None,
        ),
        (
            ActionKind::Transform(TransformKind::Remux),
            "transform",
            Some("remux"),
        ),
        (
            ActionKind::Transform(TransformKind::Segment),
            "transform",
            Some("segment"),
        ),
        (
            ActionKind::Transform(TransformKind::Transcode),
            "transform",
            Some("transcode"),
        ),
        (ActionKind::CacheUpgrade(range), "cache_upgrade", None),
        (
            ActionKind::Hedge {
                primary: ActionId::new(2),
                alternate: "https://alternate.example/media".into(),
            },
            "hedge",
            None,
        ),
        (ActionKind::Cancel(ActionId::new(3)), "cancel", None),
    ];

    for (kind, tag, transform) in cases {
        let command = PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://origin.example/media".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        };
        let value = serde_json::to_value(record(&decision("secret-post", command, kind)))
            .expect("valid test fixture");
        let kind = &value["warp_decision"]["selected"]["kind"];
        assert_eq!(kind["kind"], tag);
        if let Some(transform) = transform {
            assert_eq!(kind["transform"], transform);
        }
    }
}
