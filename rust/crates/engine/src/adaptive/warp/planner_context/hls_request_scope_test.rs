use super::request_scope::RequestScope;
use crate::adaptive::{ActionKind, ActionNode, ActionValue, HlsBootstrapStage, ResourceCost};
use crate::{ByteRange, PostId};

#[test]
fn hls_demand_capacity_cannot_be_spent_by_progressive_work() {
    let scope = RequestScope::new(1, 2, Vec::new());
    let hls = node(ActionKind::HlsBootstrap {
        stage: HlsBootstrapStage::RootManifest,
        cursor: Default::default(),
        maximum_bytes: 1024,
    });
    let progressive = node(ActionKind::FetchRange(ByteRange::new(0, 1024)));

    assert!(scope.admits(&hls, 1));
    assert!(!scope.admits(&progressive, 1));
}

fn node(kind: ActionKind) -> ActionNode {
    ActionNode::new(
        1,
        PostId::new("post"),
        kind,
        ActionValue::from_net_micros(1),
    )
    .with_resources(ResourceCost::new(1024, 0, 0, 1))
    .with_origin("https://media.example/video")
}
