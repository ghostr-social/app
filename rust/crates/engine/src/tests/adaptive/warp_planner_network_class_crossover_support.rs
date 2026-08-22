use super::network_class_support::{evidence, record, NetworkClassFixture};
use crate::adaptive::{AdaptivePlayabilityPolicy, FeedOffset, PlannerContext, ViewProbability};
use crate::origin_model::{NetworkClass, OriginModel};
use crate::tests::adaptive_support::snapshot;

const WIFI_SOURCE: &str = "https://wifi-fast.example/video.mp4";
const CELLULAR_SOURCE: &str = "https://cellular-fast.example/video.mp4";

pub(super) fn fixture(network_class: NetworkClass) -> NetworkClassFixture {
    let mut snapshot = snapshot(2, 80_000_000, 20_000, 0);
    for (candidate, source) in snapshot.candidates.iter_mut().zip(sources()) {
        candidate.feed_offset = FeedOffset::new(0);
        candidate.view_probability = ViewProbability::new(1.0).unwrap();
        candidate.origins[0].source = source.to_owned();
    }
    let base = AdaptivePlayabilityPolicy.plan(&snapshot);
    let context =
        PlannerContext::explicitly_unavailable(&snapshot).with_network_class(network_class);
    NetworkClassFixture {
        snapshot,
        base,
        origins: origins(),
        context,
    }
}

fn origins() -> OriginModel {
    let mut model = OriginModel::default();
    for sample in 0..24 {
        record(
            &mut model,
            WIFI_SOURCE,
            evidence(NetworkClass::Wifi, 80_000_000, 20, sample),
        );
        record(
            &mut model,
            WIFI_SOURCE,
            evidence(NetworkClass::Cellular, 1_000_000, 400, sample),
        );
        record(
            &mut model,
            CELLULAR_SOURCE,
            evidence(NetworkClass::Wifi, 1_000_000, 400, sample),
        );
        record(
            &mut model,
            CELLULAR_SOURCE,
            evidence(NetworkClass::Cellular, 80_000_000, 20, sample),
        );
    }
    model
}

fn sources() -> [&'static str; 2] {
    [WIFI_SOURCE, CELLULAR_SOURCE]
}
