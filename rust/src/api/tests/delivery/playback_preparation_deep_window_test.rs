use crate::api::delivery_types::FfiPlaybackPreparationReadiness as Readiness;
use crate::api::playback_preparation_stream::{projection, PreparationContext};
use crate::api::runtime::tracked_items::TrackedItems;
use crate::api::tests::delivery::playback_preparation_sparse_fixture::complete_startup;
use crate::api::tests::support::{bind_store, sized_meta, temp_store};
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_delivery::delivery_events::command_channel;
use ghostr_delivery::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::{
    AllocationPlan, ReadyReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState,
};
use ghostr_engine::PostId;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;

#[tokio::test]
async fn projects_every_certified_upcoming_asset_in_feed_order() {
    let store = temp_store("ghostr-deep-preparation");
    let tracked = TrackedItems::new();
    let cache = CacheRegistry::new();
    let startup = complete_startup(&sized_meta(16, 2_000), 16);
    let mut certificates = Vec::new();
    for id in ["next-1", "next-2", "next-3"] {
        prepare(&store, &tracked, id).await;
        certificates.push(
            StartupCertificate::issue(
                startup.clone(),
                &store
                    .media_snapshot(id)
                    .await
                    .expect("test fixture precondition must hold"),
            )
            .expect("test fixture precondition must hold"),
        );
    }
    cache.replace(["next-1", "next-2", "next-3"].into_iter().map(cached));
    let plan = AllocationPlan {
        ready_reserve: ReadyReserveEvidence {
            candidates: ["next-1", "next-2", "next-3"]
                .into_iter()
                .enumerate()
                .map(|(index, id)| ReserveCandidateEvidence {
                    post: PostId::new(id),
                    state: match index {
                        1 => ReserveCandidateState::Structural {
                            startup: startup.clone(),
                        },
                        _ => ReserveCandidateState::Ready {
                            startup: startup.clone(),
                        },
                    },
                })
                .collect(),
            ..Default::default()
        },
        ..Default::default()
    };
    let (delivery, mut commands) = command_channel();
    commands.publish_focused_plan_with_startups(7, None, plan, certificates);
    let context = PreparationContext {
        endpoint: "127.0.0.1:8080".to_owned(),
        store,
        capabilities: ProgressiveCapabilities::production(),
        delivery,
        tracked,
        cache,
    };

    let projected = projection::project(&context)
        .await
        .expect("preparation plan");
    let assets: Vec<_> = projected
        .upcoming
        .iter()
        .map(|asset| (asset.delivery_id.as_str(), asset.readiness))
        .collect();
    assert_eq!(
        assets,
        [
            ("next-1", Readiness::StructuralStartable),
            ("next-2", Readiness::StructuralStartable),
            ("next-3", Readiness::StructuralStartable),
        ]
    );
    assert_eq!(
        projected
            .next
            .as_ref()
            .expect("test fixture precondition must hold")
            .delivery_id,
        "next-1"
    );
}

async fn prepare(
    store: &ghostr_partial_store::partial_range_store::PartialRangeStore,
    tracked: &TrackedItems,
    id: &str,
) {
    let meta = sized_meta(16, 2_000);
    bind_store(store, id, &meta).await;
    store
        .set_total_len(id, 16)
        .await
        .expect("test fixture precondition must hold");
    store
        .write_range(id, 0, &[7; 16])
        .await
        .expect("test fixture precondition must hold");
    tracked.insert(id.to_owned(), meta);
}

fn cached(id: &str) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta: sized_meta(16, 2_000),
        status: CacheStatus::Complete,
    }
}
