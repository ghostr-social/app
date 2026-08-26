use crate::api::tests::delivery::playback_preparation_sparse_fixture::complete_startup;
use crate::api::tests::support::sized_meta;
use ghostr_delivery::delivery_events::CommandReceiver;
use ghostr_delivery::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{AllocationPlan, NextReserveEvidence};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::PartialRangeStore;

pub(super) async fn publish(commands: &mut CommandReceiver, store: &PartialRangeStore) {
    let startup = complete_startup(&sized_meta(16, 2_000), 16);
    let snapshot = store
        .media_snapshot("next")
        .await
        .expect("test fixture precondition must hold");
    let certificate = StartupCertificate::issue(startup.clone(), &snapshot)
        .expect("test fixture precondition must hold");
    let plan = AllocationPlan {
        next_reserve: NextReserveEvidence::Ready {
            post: PostId::new("next"),
            startup,
        },
        ..AllocationPlan::default()
    };
    commands.publish_focused_plan_with_startup(
        7,
        Some(PostId::new("current")),
        plan,
        Some(certificate),
    );
}
