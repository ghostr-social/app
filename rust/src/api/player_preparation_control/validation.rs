use super::{FfiPlayerPreparationReport, PlayerPreparationAuthority, PlayerPreparationContext};
use crate::api::delivery::focus_mapping::validate_post_id;
use core::fmt::{Display, Formatter};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AssetValidationError {
    Rejected,
    Unavailable,
}

struct CandidateAsset {
    post: PostId,
    snapshot: StoredMediaSnapshot,
}

pub(super) async fn validate_asset(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
) -> Result<PlayerPreparationAuthority, AssetValidationError> {
    let post = validated_post(input)?;
    let meta = context
        .tracked
        .meta(post.as_str())
        .ok_or(AssetValidationError::Rejected)?;
    match meta.delivery {
        DeliveryKind::Progressive => validate_progressive(context, input, post, &meta).await,
        DeliveryKind::Hls => validate_hls(context, input, post),
    }
}

async fn validate_progressive(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
    post: PostId,
    meta: &VideoMeta,
) -> Result<PlayerPreparationAuthority, AssetValidationError> {
    let candidate = load_candidate(context, post).await?;
    validate_binding(context, input, meta, &candidate)?;
    ensure_capability(context, input, &candidate).await?;
    let binding = candidate.binding()?.clone();
    ensure_revision(context, &candidate, &binding).await?;
    let revision = candidate.snapshot.revision();
    PlayerPreparationAuthority::try_new(candidate.post, binding, revision, &input.asset_id)
        .ok_or(AssetValidationError::Rejected)
}

fn validate_hls(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
    post: PostId,
) -> Result<PlayerPreparationAuthority, AssetValidationError> {
    let revision = parse_hls_asset_revision(&input.asset_id)?;
    let authority = context
        .segmented
        .resolve_prepared_authority(post.as_str(), &input.representation_id, revision)
        .ok_or(AssetValidationError::Rejected)?;
    PlayerPreparationAuthority::try_new_hls(authority, &input.asset_id)
        .ok_or(AssetValidationError::Rejected)
}

async fn load_candidate(
    context: &PlayerPreparationContext,
    post: PostId,
) -> Result<CandidateAsset, AssetValidationError> {
    let snapshot = context
        .store
        .media_snapshot(post.as_str())
        .await
        .map_err(|error| {
            log::debug!("Player preparation media snapshot is unavailable: {error:#}");
            AssetValidationError::Unavailable
        })?;
    Ok(CandidateAsset { post, snapshot })
}

fn validated_post(input: &FfiPlayerPreparationReport) -> Result<PostId, AssetValidationError> {
    validate_post_id(&input.post_id).map_err(|error| {
        log::debug!("Rejected player preparation post id: {error:#}");
        AssetValidationError::Rejected
    })?;
    Ok(PostId::new(input.post_id.clone()))
}

fn parse_hls_asset_revision(asset_id: &str) -> Result<u64, AssetValidationError> {
    let raw = asset_id
        .strip_prefix("hls-v1:")
        .ok_or(AssetValidationError::Rejected)?;
    require(!raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit()))?;
    let revision = raw
        .parse::<u64>()
        .map_err(|_| AssetValidationError::Rejected)?;
    require(revision > 0 && raw == revision.to_string())?;
    Ok(revision)
}

fn validate_binding(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
    meta: &VideoMeta,
    candidate: &CandidateAsset,
) -> Result<(), AssetValidationError> {
    let binding = candidate.binding()?;
    require(binding.matches_source_meta(meta))?;
    require(binding.representation().fingerprint() == input.representation_id)?;
    require(
        context
            .cache
            .matches_binding(candidate.post.as_str(), binding),
    )
}

async fn ensure_capability(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
    candidate: &CandidateAsset,
) -> Result<(), AssetValidationError> {
    require(
        context
            .capabilities
            .authorizes(
                &input.asset_id,
                candidate.post.as_str(),
                &candidate.snapshot,
            )
            .await,
    )
}

async fn ensure_revision(
    context: &PlayerPreparationContext,
    candidate: &CandidateAsset,
    binding: &RepresentationBinding,
) -> Result<(), AssetValidationError> {
    let current = context
        .store
        .stream_is_current(
            candidate.post.as_str(),
            Some(binding),
            candidate.snapshot.revision(),
        )
        .await
        .map_err(|error| {
            log::debug!("Player preparation revision check is unavailable: {error:#}");
            AssetValidationError::Unavailable
        })?;
    require(current)
}

impl CandidateAsset {
    fn binding(&self) -> Result<&RepresentationBinding, AssetValidationError> {
        self.snapshot
            .binding()
            .ok_or(AssetValidationError::Rejected)
    }
}

fn require(valid: bool) -> Result<(), AssetValidationError> {
    valid.then_some(()).ok_or(AssetValidationError::Rejected)
}

impl Display for AssetValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Rejected => formatter.write_str("player preparation authority was rejected"),
            Self::Unavailable => formatter.write_str("player preparation authority is unavailable"),
        }
    }
}

impl core::error::Error for AssetValidationError {}
