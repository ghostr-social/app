use super::{context, MeasuredTraffic, ObservationTiming};
use crate::chunk::traffic::ChunkTraffic;
use core::time::Duration;
use ghostr_engine::origin_model::{
    MediaClass, NetworkClass, OriginAttemptContext, OriginAttemptProfile, OriginRequestProfile,
    RequestMethod,
};

#[test]
fn completion_does_not_rekey_the_request_start_context() {
    let profile = OriginAttemptProfile::new(OriginRequestProfile::new(
        RequestMethod::PrefixGet,
        16,
        MediaClass::Unknown,
    ));
    let mut sink = LiveNetwork(NetworkClass::Cellular);
    let mut traffic = MeasuredTraffic::new(&mut sink, NetworkClass::Wifi, profile);
    traffic.concurrency(4);
    traffic.capture_request_start(21_599_999);
    let measured = traffic.measurements();
    let completion = ObservationTiming {
        at_ms: 21_600_001,
        elapsed: Duration::from_millis(2),
    };
    let expected = OriginAttemptContext::new(profile, NetworkClass::Cellular, 4, 21_599_999);

    assert_eq!(context(&measured, completion), expected.request_context());
}

struct LiveNetwork(NetworkClass);

impl ChunkTraffic for LiveNetwork {
    fn current_network_class(&mut self) -> Option<NetworkClass> {
        Some(self.0)
    }

    fn opened(&mut self, _ttfb: Duration) {}

    fn wrote(&mut self, _bytes: u64) {}
}
