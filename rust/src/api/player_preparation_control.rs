//! Authority-fenced native-player evidence admitted to WARP planning.

use crate::api::delivery_types::{
    FfiPlayerPreparationDisposition, FfiPlayerPreparationReport, FfiPlayerPreparationState,
};
use crate::api::runtime::registry;
use crate::api::runtime::tracked_items::TrackedItems;
use flutter_rust_bridge::frb;
use ghostr_delivery::cache_registry::CacheRegistry;

use ghostr_delivery::delivery_events::{
    DeliveryHandle, PlayerPreparationAuthority, PlayerPreparationDisposition,
};
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;

mod mapping;
mod validation;
use mapping::{map_followup, map_initial};
use validation::{validate_asset, AssetValidationError};

pub(crate) struct PlayerPreparationContext {
    pub(super) store: Arc<PartialRangeStore>,
    pub(super) capabilities: ProgressiveCapabilities,
    pub(super) delivery: DeliveryHandle,
    pub(super) tracked: TrackedItems,
    pub(super) cache: CacheRegistry,
}

#[frb]
pub async fn ffi_report_player_preparation(
    input: FfiPlayerPreparationReport,
) -> FfiPlayerPreparationDisposition {
    let Ok(engine) = registry::engine() else {
        return FfiPlayerPreparationDisposition::Closed;
    };
    let progressive = engine.gateway.progressive();
    let context = PlayerPreparationContext {
        store: progressive.store.clone(),
        capabilities: progressive.capabilities.clone(),
        delivery: engine.gateway.delivery(),
        tracked: engine.tracked.clone(),
        cache: progressive.cache.clone(),
    };
    confirm_player_preparation(&context, input).await
}

pub(crate) async fn confirm_player_preparation(
    context: &PlayerPreparationContext,
    input: FfiPlayerPreparationReport,
) -> FfiPlayerPreparationDisposition {
    let initial = input.sequence == 1 && input.state == FfiPlayerPreparationState::Initializing;
    if initial {
        return confirm_initial(context, input).await;
    }
    let Ok(report) = map_followup(&input) else {
        return FfiPlayerPreparationDisposition::Rejected;
    };
    context
        .delivery
        .confirm_player_preparation_followup(report)
        .await
        .into()
}

async fn confirm_initial(
    context: &PlayerPreparationContext,
    input: FfiPlayerPreparationReport,
) -> FfiPlayerPreparationDisposition {
    let Ok(probe) = map_followup(&input) else {
        return FfiPlayerPreparationDisposition::Rejected;
    };
    if let Some(disposition) = context.delivery.player_preparation_disposition(&probe) {
        return disposition.into();
    }
    let admission = context.delivery.player_preparation_admission();
    let authority = match validate_asset(context, &input).await {
        Ok(authority) => authority,
        Err(AssetValidationError::Rejected) => {
            return FfiPlayerPreparationDisposition::Rejected;
        }
        Err(AssetValidationError::Unavailable) => {
            return FfiPlayerPreparationDisposition::NotAdmitted;
        }
    };
    let Ok(report) = map_initial(&input, authority) else {
        return FfiPlayerPreparationDisposition::Rejected;
    };
    context
        .delivery
        .confirm_player_preparation_initial(admission, report)
        .await
        .into()
}

impl From<PlayerPreparationDisposition> for FfiPlayerPreparationDisposition {
    fn from(value: PlayerPreparationDisposition) -> Self {
        match value {
            PlayerPreparationDisposition::Applied => Self::Applied,
            PlayerPreparationDisposition::Duplicate => Self::Duplicate,
            PlayerPreparationDisposition::Stale => Self::Stale,
            PlayerPreparationDisposition::MissingInitial => Self::MissingInitial,
            PlayerPreparationDisposition::Rejected => Self::Rejected,
            PlayerPreparationDisposition::Saturated => Self::Saturated,
            PlayerPreparationDisposition::Unavailable => Self::Unavailable,
            PlayerPreparationDisposition::Closed => Self::Closed,
            PlayerPreparationDisposition::NotAdmitted => Self::NotAdmitted,
        }
    }
}

#[cfg(test)]
#[path = "player_preparation_control_axiom_test.rs"]
pub(crate) mod axiom_test_support;
