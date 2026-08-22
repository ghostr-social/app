//! Authority-fenced native-player evidence admitted to WARP planning.

use crate::api::delivery::focus_mapping::validate_post_id;
use crate::api::delivery_types::{FfiPlayerPreparationReport, FfiPlayerPreparationState};
use crate::api::runtime::registry;
use crate::api::runtime::tracked_items::TrackedItems;
use anyhow::{ensure, Context};
use flutter_rust_bridge::frb;
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::delivery_events::{
    DeliveryHandle, PlayerPreparationAttempt, PlayerPreparationAuthority, PlayerPreparationIngress,
    PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState,
};
use ghostr_engine::{DeliveryKind, PostId};
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, StoredMediaSnapshot};
use std::sync::Arc;

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
    let authority = validate_asset(context, &input).await?;
    let report = map_report(input, authority)?;
    admit(context.delivery.report_player_preparation(report))
}

struct CandidateAsset {
    post: PostId,
    snapshot: StoredMediaSnapshot,
}

async fn load_candidate(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
) -> anyhow::Result<CandidateAsset> {
    validate_post_id(&input.post_id)?;
    let post = PostId::new(input.post_id.clone());
    let meta = context
        .tracked
        .meta(post.as_str())
        .context("player preparation post is not tracked")?;
    ensure!(
        meta.delivery == DeliveryKind::Progressive,
        "player preparation is not progressive"
    );
    let snapshot = context.store.media_snapshot(post.as_str()).await?;
    let binding = snapshot
        .binding()
        .context("player preparation has no stored representation")?;
    ensure!(
        binding.matches_or_derives_from(&meta),
        "tracked representation is stale"
    );
    ensure!(
        binding.representation().fingerprint() == input.representation_id,
        "player representation is stale"
    );
    ensure!(
        context.cache.matches_binding(post.as_str(), binding),
        "player cache authority is stale"
    );
    Ok(CandidateAsset { post, snapshot })
}

async fn validate_asset(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
) -> anyhow::Result<PlayerPreparationAuthority> {
    let candidate = load_candidate(context, input).await?;
    ensure!(
        context
            .capabilities
            .authorizes(
                &input.asset_id,
                candidate.post.as_str(),
                &candidate.snapshot
            )
            .await,
        "player asset capability is stale"
    );
    let binding = candidate
        .snapshot
        .binding()
        .context("stored representation disappeared")?;
    ensure!(
        context
            .store
            .stream_is_current(
                candidate.post.as_str(),
                Some(binding),
                candidate.snapshot.revision()
            )
            .await,
        "player content revision is stale"
    );
    PlayerPreparationAuthority::try_new(
        candidate.post,
        binding.clone(),
        candidate.snapshot.revision(),
    )
    .context("player preparation authority is inconsistent")
}

fn map_report(
    input: FfiPlayerPreparationReport,
    authority: PlayerPreparationAuthority,
) -> anyhow::Result<PlayerPreparationReport> {
    let attempt = PlayerPreparationAttempt::try_new(
        input.player_capability_generation,
        input.client_epoch,
        input.attempt_generation,
    )
    .context("player preparation generations must be positive")?;
    let observation = PlayerPreparationObservation::try_new(
        input.state.into(),
        input.failure_kind,
        input.observed_monotonic_us,
    )
    .context("player preparation failure kind is invalid")?;
    let report = PlayerPreparationReport::try_new(authority, attempt, input.sequence, observation)
        .context("player preparation sequence must be positive")?;
    Ok(report)
}

fn admit(admission: PlayerPreparationIngress) -> anyhow::Result<()> {
    match admission {
        PlayerPreparationIngress::Accepted | PlayerPreparationIngress::Stale => Ok(()),
        PlayerPreparationIngress::Saturated => {
            anyhow::bail!("player preparation mailbox is saturated")
        }
        PlayerPreparationIngress::Closed => anyhow::bail!("delivery manager is unavailable"),
    }
}

impl From<FfiPlayerPreparationState> for PlayerPreparationState {
    fn from(value: FfiPlayerPreparationState) -> Self {
        match value {
            FfiPlayerPreparationState::Initializing => Self::Initializing,
            FfiPlayerPreparationState::Initialized => Self::Initialized,
            FfiPlayerPreparationState::FirstFrameRendered => Self::FirstFrameRendered,
            FfiPlayerPreparationState::Failed => Self::Failed,
            FfiPlayerPreparationState::Released => Self::Released,
        }
    }
}
