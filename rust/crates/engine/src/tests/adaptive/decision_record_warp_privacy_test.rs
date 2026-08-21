use super::decision_record_warp_test_support::{allocation, decision, record};
use crate::adaptive::{ActionKind, PlannerCommand, RetrievalRequest};
use crate::{ActionId, ByteRange, PostId};

#[test]
fn every_source_bearing_warp_command_is_privacy_transformed_field_by_field() {
    let source = "https://private.example/media.mp4?cap=raw";
    let alternate = "https://alternate.example/video.mp4?token=secret";
    let transfer = allocation(
        source,
        RetrievalRequest::FetchRange {
            bytes: ByteRange::new(0, 64),
            promotion: None,
        },
    );
    let cases = [
        decision(
            "secret-post",
            PlannerCommand::ProbeHead {
                post: PostId::new("secret-post"),
                source: source.into(),
            },
            ActionKind::Head,
        ),
        decision(
            "secret-post",
            PlannerCommand::Transfer(transfer.clone()),
            ActionKind::FetchRange(ByteRange::new(0, 64)),
        ),
        decision(
            "secret-post",
            PlannerCommand::Hedge {
                primary: ActionId::new(4),
                transfer,
            },
            ActionKind::Hedge {
                primary: ActionId::new(4),
                alternate: alternate.into(),
            },
        ),
    ];

    for decision in cases {
        let json = serde_json::to_string(&record(&decision)).unwrap();
        for secret in [
            "secret-post",
            "private.example",
            "cap=raw",
            "alternate.example",
            "token=secret",
        ] {
            assert!(!json.contains(secret), "leaked {secret}: {json}");
        }
    }
}
