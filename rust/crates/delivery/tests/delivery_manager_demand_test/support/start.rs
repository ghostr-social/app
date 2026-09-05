use super::{next_request, ControlledOrigin, DemandScenario, POST};
use crate::delivery_fixture::items::{focus_now, sized_item};
use crate::fixture::constrained_harness;
use crate::{CHUNK, MEBIBYTE};
use ghostr_engine::adaptive::MEDIA_BOOTSTRAP_PROBE_BYTES;

impl DemandScenario {
    pub async fn start() -> Self {
        let mut origin = ControlledOrigin::serve(2 * MEBIBYTE).await;
        let harness = constrained_harness("ghostr-delivery-demand", MEBIBYTE);
        let item = sized_item(POST, &origin.url, 2 * MEBIBYTE, 512_000);
        harness.handle.update_focus(focus_now(vec![item], 0, 0));
        let initial = next_request(&mut origin).await;
        assert_eq!(initial.range.start, 0, "bootstrap begins at zero");
        assert!(
            initial.range.end >= MEDIA_BOOTSTRAP_PROBE_BYTES,
            "bootstrap includes discovery bytes"
        );
        assert!(
            initial.range.end <= CHUNK,
            "bootstrap respects the chunk cap"
        );
        let revision = harness.handle.latest_plan().expect("initial plan").revision;
        Self {
            origin,
            harness,
            initial: Some(initial),
            initial_sent: 0,
            revision,
        }
    }
}
