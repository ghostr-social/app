use super::playback_preparation_current_focus_fixture::{focus_other, seed_other};
use super::playback_preparation_current_lifecycle_fixture::{context, CurrentLifecycleFixture};
use crate::api::delivery_types::{
    FfiPlaybackPreparationAsset, FfiPlaybackPreparationReadiness as Readiness,
};
use crate::api::playback_preparation_stream::{projection, PreparationContext};
use core::time::Duration;
use ghostr_delivery::delivery_events::FocusAdmission;
use tokio::time::{timeout_at, Instant};

const FOCUS_WAIT_LIMIT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn a_new_current_post_does_not_inherit_the_old_first_frame() {
    let mut fixture = CurrentLifecycleFixture::start().await;
    fixture.render_first_frame().await;
    let manager_context = &fixture.manager.context;
    let meta = manager_context
        .tracked
        .meta("clip")
        .expect("seeded metadata");
    seed_other(&fixture, &meta).await;
    let admission = manager_context.delivery.update_focus(focus_other(&meta));
    assert_eq!(admission, FocusAdmission::Accepted);
    let current = wait_for_other_projection(&fixture, &meta).await;
    assert_eq!(current.readiness, Readiness::Preparing);
    fixture.shutdown().await;
}

async fn wait_for_other_projection(
    fixture: &CurrentLifecycleFixture,
    meta: &ghostr_engine::VideoMeta,
) -> FfiPlaybackPreparationAsset {
    let context = context(&fixture.manager);
    let notifier = context.delivery.plan_notifier();
    let deadline = Instant::now() + FOCUS_WAIT_LIMIT;
    let mut reseeded = false;
    loop {
        let changed = notifier.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        let focused = focused_with_old_claim(&context);
        if focused && !reseeded {
            seed_other(fixture, meta).await;
            reseeded = true;
        }
        if focused {
            if let Some(current) = projected_other(&context).await {
                return current;
            }
        }
        if timeout_at(deadline, changed).await.is_err() {
            return deadline_boundary(&context, fixture, meta).await;
        }
    }
}

fn focused_with_old_claim(context: &PreparationContext) -> bool {
    context.delivery.latest_plan().is_some_and(|plan| {
        plan.current
            .as_ref()
            .is_some_and(|post| post.as_str() == "other")
            && plan
                .player_preparations
                .iter()
                .any(|claim| claim.post().as_str() == "clip")
    })
}

async fn projected_other(context: &PreparationContext) -> Option<FfiPlaybackPreparationAsset> {
    projection::project(context)
        .await?
        .current
        .filter(|asset| asset.delivery_id == "other")
}

async fn deadline_boundary(
    context: &PreparationContext,
    fixture: &CurrentLifecycleFixture,
    meta: &ghostr_engine::VideoMeta,
) -> FfiPlaybackPreparationAsset {
    if focused_with_old_claim(context) {
        seed_other(fixture, meta).await;
        if let Some(projected) = projected_other(context).await {
            return projected;
        }
    }
    let latest = context
        .delivery
        .latest_plan()
        .map(|plan| (plan.revision, plan.current, plan.player_preparations.len()));
    let projected = projection::project(context).await.and_then(|plan| {
        plan.current
            .map(|asset| (asset.delivery_id, asset.readiness))
    });
    panic!("manager focus deadline; latest={latest:?}; projected={projected:?}")
}
