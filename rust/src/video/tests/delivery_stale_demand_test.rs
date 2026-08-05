use crate::engine::tiers::DemandSignals;
use crate::engine::{ByteRange, PostId};
use crate::video::delivery_reconcile::resolve_demand;
use crate::video::playback_demand::DemandSignal;
use std::collections::HashMap;

#[test]
fn demand_for_a_non_playing_post_is_consumed_as_stale() {
    let post = PostId::new("stale");
    let mut pending = Some(signal(post.clone()));

    let demand = resolve_demand(&mut pending, None, &HashMap::new());

    assert_eq!(demand, DemandSignals::default());
    assert_eq!(pending, None);
}

#[test]
fn demand_for_bytes_already_present_is_consumed_as_stale() {
    let post = PostId::new("playing");
    let mut pending = Some(signal(post.clone()));
    let present = HashMap::from([(post.clone(), vec![ByteRange::new(0, 16)])]);

    let demand = resolve_demand(&mut pending, Some(&post), &present);

    assert_eq!(demand, DemandSignals::default());
    assert_eq!(pending, None);
}

fn signal(post: PostId) -> DemandSignal {
    DemandSignal {
        post,
        range: ByteRange::new(4, 8),
    }
}
