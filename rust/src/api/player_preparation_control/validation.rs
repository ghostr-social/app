use super::{FfiPlayerPreparationReport, PlayerPreparationAuthority, PlayerPreparationContext};
use crate::api::delivery::focus_mapping::validate_post_id;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::fmt::{Display, Formatter};

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
    let candidate = load_candidate(context, input).await?;
    ensure_capability(context, input, &candidate).await?;
    let binding = candidate.binding()?.clone();
    ensure_revision(context, &candidate, &binding).await?;
    let revision = candidate.snapshot.revision();
    PlayerPreparationAuthority::try_new(candidate.post, binding, revision, &input.asset_id)
        .ok_or(AssetValidationError::Rejected)
}

async fn load_candidate(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
) -> Result<CandidateAsset, AssetValidationError> {
    validate_post_id(&input.post_id).map_err(|_| AssetValidationError::Rejected)?;
    let post = PostId::new(input.post_id.clone());
    let meta = context
        .tracked
        .meta(post.as_str())
        .ok_or(AssetValidationError::Rejected)?;
    require(meta.delivery == DeliveryKind::Progressive)?;
    let snapshot = context
        .store
        .media_snapshot(post.as_str())
        .await
        .map_err(|_| AssetValidationError::Unavailable)?;
    let candidate = CandidateAsset { post, snapshot };
    validate_binding(context, input, &meta, &candidate)?;
    Ok(candidate)
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
        .map_err(|_| AssetValidationError::Unavailable)?;
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
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected => formatter.write_str("player preparation authority was rejected"),
            Self::Unavailable => formatter.write_str("player preparation authority is unavailable"),
        }
    }
}

impl std::error::Error for AssetValidationError {}
