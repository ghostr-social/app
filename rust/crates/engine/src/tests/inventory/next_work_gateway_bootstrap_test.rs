use crate::playback::PLAYBACK_SLICE_BYTES;
use crate::tests::scheduling_support::{focus_at, WorkBench};
use crate::tests::support::progressive_meta;
use crate::tiers::Tier;
use crate::{ByteRange, PostId};

const TOTAL: u64 = 370_912;

#[test]
fn gateway_demand_keeps_foreground_depth_and_bounds_the_ahead_grant() {
    let mut bench = WorkBench::new();
    for post in ["current", "next"] {
        bench.catalog.upsert(
            PostId::new(post),
            progressive_meta(Some(TOTAL), Some(3_000)),
        );
    }
    bench.focus = focus_at(&["current", "next"], 0, 0);
    bench.demand.gateway_demand = true;

    let requests = bench.run();

    assert_eq!(requests[0].chunk.range, ByteRange::new(0, TOTAL));
    assert_eq!(requests[0].tier, Tier::T0PlaybackEmergency);
    assert_eq!(requests[1].chunk.post, PostId::new("next"));
    assert_eq!(
        requests[1].chunk.range,
        ByteRange::new(0, PLAYBACK_SLICE_BYTES)
    );
    assert!(requests[1..]
        .iter()
        .all(|request| request.tier == Tier::T2Startability));

    bench.present.insert(
        PostId::new("current"),
        vec![ByteRange::new(0, PLAYBACK_SLICE_BYTES)],
    );
    let requests = bench.run();
    assert_eq!(requests[0].chunk.post, PostId::new("current"));
    assert_eq!(
        requests[0].chunk.range,
        ByteRange::new(PLAYBACK_SLICE_BYTES, TOTAL)
    );
    assert_eq!(requests[0].tier, Tier::T0PlaybackEmergency);
}
