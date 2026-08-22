use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, HardBudget, HlsBootstrapStage, ResourceCost,
};
use crate::PostId;

#[test]
fn an_hls_block_cannot_spend_beyond_its_hard_network_envelope() {
    let node = ActionNode::new(
        1,
        PostId::new("stream"),
        ActionKind::HlsBootstrap {
            stage: HlsBootstrapStage::Initialization,
            cursor: Default::default(),
            maximum_bytes: 256 * 1024,
        },
        ActionValue::from_net_micros(1),
    )
    .with_resources(ResourceCost::new(256 * 1024, 256 * 1024, 0, 1))
    .with_origin("https://media.example/init.mp4");

    let short = HardBudget::new(ResourceCost::new(256 * 1024 - 1, u64::MAX, 0, 1), 1);
    let exact = HardBudget::new(ResourceCost::new(256 * 1024, u64::MAX, 0, 1), 1);

    assert!(!short.allows_action(&node));
    assert!(exact.allows_action(&node));
    assert_eq!(node.authorized_resources().network_bytes, 256 * 1024);
}
