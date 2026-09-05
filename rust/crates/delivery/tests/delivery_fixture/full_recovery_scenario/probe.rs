use super::Scenario;
use crate::delivery_fixture::full_recovery_origin::{ObservedRequest, TRIAL_BYTES};
use crate::delivery_fixture::items::{focus_now, sized_item};

impl Scenario {
    pub(super) async fn start_probe(&mut self) -> ObservedRequest {
        self.harness.handle.update_focus(focus_now(
            vec![sized_item(
                "probe",
                &self.probe_url,
                TRIAL_BYTES as u64,
                10_000,
            )],
            0,
            0,
        ));
        let probe = self.origin.next_within("capped probe").await;
        assert_eq!(probe.method.as_str(), "GET", "bootstrap uses GET");
        assert_eq!(probe.path, "/probe.mp4", "bootstrap uses probe source");
        assert_eq!(
            probe.range.as_deref(),
            Some("bytes=0-65535"),
            "bootstrap range stays bounded"
        );
        assert_eq!(
            probe.encoding.as_deref(),
            Some("identity"),
            "bootstrap uses identity encoding"
        );
        probe
    }
}
