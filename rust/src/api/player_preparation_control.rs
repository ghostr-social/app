//! Authority-fenced native-player evidence admitted to WARP planning.

use crate::api::delivery_types::{FfiPlayerPreparationReport, FfiPlayerPreparationState};
use crate::api::runtime::registry;
use crate::api::runtime::tracked_items::TrackedItems;
use flutter_rust_bridge::frb;
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::delivery_events::{
    DeliveryHandle, PlayerPreparationAuthority, PlayerPreparationIngress,
};
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;

mod mapping;
mod validation;
use mapping::{map_followup, map_initial};
use validation::validate_asset;

pub(crate) struct PlayerPreparationContext {
    pub(crate) store: Arc<PartialRangeStore>,
    pub(crate) capabilities: ProgressiveCapabilities,
    pub(crate) delivery: DeliveryHandle,
    pub(crate) tracked: TrackedItems,
    pub(crate) cache: CacheRegistry,
}

#[frb]
pub async fn ffi_report_player_preparation(
    input: FfiPlayerPreparationReport,
) -> anyhow::Result<()> {
    let engine = registry::engine()?;
    let progressive = engine.gateway.progressive();
    let context = PlayerPreparationContext {
        store: progressive.store.clone(),
        capabilities: progressive.capabilities.clone(),
        delivery: engine.gateway.delivery(),
        tracked: engine.tracked.clone(),
        cache: progressive.cache.clone(),
    };
    report_player_preparation(&context, input).await
}

pub(crate) async fn report_player_preparation(
    context: &PlayerPreparationContext,
    input: FfiPlayerPreparationReport,
) -> anyhow::Result<()> {
    let initial = input.sequence == 1 && input.state == FfiPlayerPreparationState::Initializing;
    if initial {
        report_initial(context, input).await
    } else {
        report_followup(context, input)
    }
}

async fn report_initial(
    context: &PlayerPreparationContext,
    input: FfiPlayerPreparationReport,
) -> anyhow::Result<()> {
    let admission = context.delivery.player_preparation_admission();
    let authority = validate_asset(context, &input).await?;
    let report = map_initial(&input, authority)?;
    admit(
        context
            .delivery
            .report_player_preparation_initial(admission, report),
    )
}

fn report_followup(
    context: &PlayerPreparationContext,
    input: FfiPlayerPreparationReport,
) -> anyhow::Result<()> {
    admit(
        context
            .delivery
            .report_player_preparation_followup(map_followup(input)?),
    )
}

fn admit(admission: PlayerPreparationIngress) -> anyhow::Result<()> {
    match admission {
        PlayerPreparationIngress::Accepted | PlayerPreparationIngress::Stale => Ok(()),
        PlayerPreparationIngress::Rejected => {
            anyhow::bail!("player preparation attempt was not admitted")
        }
        PlayerPreparationIngress::Saturated => {
            anyhow::bail!("player preparation mailbox is saturated")
        }
        PlayerPreparationIngress::Closed => anyhow::bail!("delivery manager is unavailable"),
    }
}
