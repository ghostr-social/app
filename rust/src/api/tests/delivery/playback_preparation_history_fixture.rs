use super::playback_preparation_current_focus_fixture::{focus_other, seed_other};
use super::playback_preparation_current_lifecycle_fixture::{context, CurrentLifecycleFixture};
use crate::api::delivery_types::FfiPlaybackPreparationAsset;
use crate::api::playback_preparation_stream::projection;
use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryHandle, FocusAdmission, PlanEvidence};

pub(super) async fn focused_history() -> CurrentLifecycleFixture {
    let mut fixture = CurrentLifecycleFixture::start().await;
    fixture.render_first_frame().await;
    let meta = fixture
        .manager
        .context
        .tracked
        .meta("clip")
        .expect("metadata");
    seed_other(&fixture, &meta).await;
    let delivery = &fixture.manager.context.delivery;
    assert_eq!(
        delivery.update_focus(focus_other(&meta)),
        FocusAdmission::Accepted
    );
    wait_for_plan(delivery, |plan| {
        plan.current
            .as_ref()
            .is_some_and(|post| post.as_str() == "other")
    })
    .await;
    fixture
}

pub(super) async fn previous_asset(
    fixture: &CurrentLifecycleFixture,
) -> Option<FfiPlaybackPreparationAsset> {
    projection::project(&context(&fixture.manager))
        .await?
        .upcoming
        .into_iter()
        .find(|asset| asset.delivery_id == "clip")
}

pub(super) async fn wait_for_plan(
    delivery: &DeliveryHandle,
    accepts: impl Fn(&PlanEvidence) -> bool,
) -> PlanEvidence {
    let notifier = delivery.plan_notifier();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(plan) = delivery.latest_plan().filter(&accepts) {
                return plan;
            }
            changed.await;
        }
    })
    .await
    .expect("focus publication deadline")
}
