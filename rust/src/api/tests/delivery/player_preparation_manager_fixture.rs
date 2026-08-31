use super::player_preparation_manager_authority::SeededAuthority;
use super::player_preparation_manager_environment;
use crate::api::delivery_types::{FfiPlayerPreparationReport, FfiPlayerPreparationState};
use crate::api::player_preparation_control::PlayerPreparationContext;
use core::time::Duration;
use ghostr_delivery::delivery_events::{DeliveryHandle, FocusAdmission};
use ghostr_delivery::playback_demand::DemandSender;
use ghostr_engine::adaptive::DiscoveryDemand;
use std::path::PathBuf;
use tokio::sync::watch;
use tokio::time::timeout;

const MANAGER_SHUTDOWN_LIMIT: Duration = Duration::from_secs(30);

pub(super) struct ProductionManagerFixture {
    pub(super) context: PlayerPreparationContext,
    pub(super) input: FfiPlayerPreparationReport,
    demand: DemandSender,
    discovery: watch::Receiver<DiscoveryDemand>,
    stats_root: PathBuf,
}

impl ProductionManagerFixture {
    pub(super) async fn seeded() -> Self {
        let authority = SeededAuthority::new().await;
        let (delivery, demand, discovery, stats_root) =
            player_preparation_manager_environment::start(
                std::sync::Arc::clone(&authority.store),
                authority.cache.clone(),
            );
        assert_eq!(
            delivery.update_focus(authority.focus()),
            FocusAdmission::Accepted
        );
        let input = preparation_input(&authority);
        let context = player_context(&authority, delivery);
        Self {
            context,
            input,
            demand,
            discovery,
            stats_root,
        }
    }

    pub(super) async fn shutdown(self) {
        let Self {
            context,
            demand,
            mut discovery,
            stats_root,
            ..
        } = self;
        drop(demand);
        drop(context);
        let closed = timeout(MANAGER_SHUTDOWN_LIMIT, async {
            while discovery.changed().await.is_ok() {}
        })
        .await;
        if closed.is_err() {
            let demand = *discovery.borrow();
            let channel = discovery.has_changed();
            let root_exists = stats_root.exists();
            panic!(
                "production manager shutdown; demand={demand:?}; channel={channel:?}; \
                 stats_root_exists={root_exists}"
            );
        }
        std::fs::remove_dir_all(stats_root).expect("test fixture precondition must hold");
    }
}

fn player_context(
    authority: &SeededAuthority,
    delivery: DeliveryHandle,
) -> PlayerPreparationContext {
    PlayerPreparationContext {
        store: std::sync::Arc::clone(&authority.store),
        capabilities: authority.capabilities.clone(),
        delivery,
        tracked: authority.tracked.clone(),
        cache: authority.cache.clone(),
        segmented: Default::default(),
    }
}

fn preparation_input(authority: &SeededAuthority) -> FfiPlayerPreparationReport {
    FfiPlayerPreparationReport {
        post_id: "clip".to_owned(),
        representation_id: authority.representation.clone(),
        asset_id: authority.asset.clone(),
        player_capability_generation: 1,
        client_epoch: 2,
        attempt_generation: 3,
        sequence: 1,
        state: FfiPlayerPreparationState::Initializing,
        failure_kind: None,
        observed_monotonic_us: 5,
    }
}
