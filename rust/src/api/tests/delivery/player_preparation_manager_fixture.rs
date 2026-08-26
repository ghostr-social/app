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
        timeout(Duration::from_secs(2), async move {
            while discovery.changed().await.is_ok() {}
        })
        .await
        .expect("production manager shutdown");
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
