mod active_control_rescue_test;
mod conflicting_rescue_test;
mod cpu_rescue_test;
mod hls_network_envelope_test;
mod rescue_auction_test;
mod rescue_charge_test;
mod rescue_path_feasibility_test;
mod same_authority_rescue_test;
mod segmented_storage_test;
mod serial_rescue_request_test;
mod zero_request_rescue_test;

use crate::adaptive::{ActionKind, ActionNode, ActionValue, ResourceCost};
use crate::{ByteRange, PostId};

fn request(id: u16, storage: u64) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new(format!("p{id}")),
        ActionKind::FetchRange(ByteRange::new(u64::from(id), u64::from(id) + 1)),
        ActionValue::from_net_micros(1),
    )
    .with_resources(ResourceCost::new(1, storage, 0, 1))
    .with_origin(format!("https://p{id}.example/media"))
}
