use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, HardBudget, HlsBootstrapStage, ResourceCost,
};
use crate::PostId;

const MIB: u64 = 1024 * 1024;

#[test]
fn expected_hls_cost_cannot_weaken_the_hard_object_envelope() {
    let node = ActionNode::new(
        1,
        PostId::new("stream"),
        ActionKind::HlsBootstrap {
            stage: HlsBootstrapStage::Initialization,
            maximum_bytes: 8 * MIB,
        },
        ActionValue::from_net_micros(1),
    )
    .with_resources(ResourceCost::new(256 * 1024, 8 * MIB, 0, 1))
    .with_origin("https://media.example/init.mp4");

    let short = HardBudget::new(ResourceCost::new(7 * MIB, u64::MAX, 0, 1), 1);
    let exact = HardBudget::new(ResourceCost::new(8 * MIB, u64::MAX, 0, 1), 1);

    assert!(!short.allows_action(&node));
    assert!(exact.allows_action(&node));
    assert_eq!(node.authorized_resources().network_bytes, 8 * MIB);
}
