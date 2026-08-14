use crate::playback_demand::{demand_channel, DemandState};
use ghostr_engine::{ByteRange, PostId};

#[test]
fn repeating_the_same_range_does_not_emit_another_lease_state() {
    let (sender, mut receiver) = demand_channel();
    let mut consumer = sender.consumer(PostId::new("current"), None);
    let range = ByteRange::new(0, 16);

    consumer.demand(range);
    assert!(matches!(receiver.try_recv(), Ok(DemandState::Blocked(_))));
    consumer.demand(range);
    assert!(receiver.try_recv().is_err());
}
