use crate::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use crate::delivery_fixture::items::candidate;
use core::time::Duration;
use ghostr_delivery::delivery_events::{
    DeliveryFocus, DeliveryHandle, FocusGeneration, FocusItem, FocusTransition,
};
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord, RecordedWarpCommand};
use ghostr_engine::PostId;

pub fn canonical_focus(origin: &ControlledOrigin, total: u64) -> DeliveryFocus {
    DeliveryFocus {
        items: vec![FocusItem {
            post: PostId::new("current"),
            meta: candidate(
                "current",
                &origin.url_for("canonical-current"),
                Some(total),
                4,
            )
            .meta,
        }],
        previews: Vec::new(),
        current_index: 0,
        watch_ms: 0,
        generation: FocusGeneration::try_new(1).expect("generation"),
        transition: FocusTransition::RosterChange,
        rescue: None,
    }
}

pub fn is_cancel(record: &DecisionRecord) -> bool {
    matches!(
        record.eventual_outcome,
        DecisionOutcome::Succeeded { bytes: 0, .. }
    ) && matches!(
        record
            .warp_decision
            .as_ref()
            .and_then(|decision| decision.selected.as_ref()),
        Some(selected) if matches!(selected.command, RecordedWarpCommand::Cancel { .. })
    )
}

pub async fn next_request(
    label: &str,
    origin: &mut ControlledOrigin,
    handle: &DeliveryHandle,
) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(5), origin.next())
        .await
        .unwrap_or_else(|_| panic!("{label} request starts; latest={:#?}", handle.latest_plan()))
}

pub async fn wait_cancelled(request: &ActiveRequest) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while request.is_open() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("far handoff cancels");
}
