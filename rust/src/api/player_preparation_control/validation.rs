use super::{FfiPlayerPreparationReport, PlayerPreparationAuthority, PlayerPreparationContext};
use crate::api::delivery::focus_mapping::validate_post_id;
use anyhow::{ensure, Context};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;

struct CandidateAsset {
    post: PostId,
    snapshot: StoredMediaSnapshot,
}

pub(super) async fn validate_asset(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
) -> anyhow::Result<PlayerPreparationAuthority> {
    let candidate = load_candidate(context, input).await?;
    ensure_capability(context, input, &candidate).await?;
    let binding = candidate.binding()?.clone();
    ensure_revision(context, &candidate, &binding).await?;
    let revision = candidate.snapshot.revision();
    PlayerPreparationAuthority::try_new(candidate.post, binding, revision, &input.asset_id)
        .context("player preparation authority is inconsistent")
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
    let candidate = CandidateAsset { post, snapshot };
    validate_binding(context, input, &meta, &candidate)?;
    Ok(candidate)
}

fn validate_binding(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
    meta: &VideoMeta,
    candidate: &CandidateAsset,
) -> anyhow::Result<()> {
    let binding = candidate.binding()?;
    ensure!(
        binding.matches_or_derives_from(meta),
        "stale representation"
    );
    ensure!(
        binding.representation().fingerprint() == input.representation_id,
        "player representation is stale"
    );
    ensure!(
        context
            .cache
            .matches_binding(candidate.post.as_str(), binding),
        "player cache authority is stale"
    );
    Ok(())
}

async fn ensure_capability(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
    candidate: &CandidateAsset,
) -> anyhow::Result<()> {
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
    Ok(())
}

async fn ensure_revision(
    context: &PlayerPreparationContext,
    candidate: &CandidateAsset,
    binding: &RepresentationBinding,
) -> anyhow::Result<()> {
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
    Ok(())
}

impl CandidateAsset {
    fn binding(&self) -> anyhow::Result<&RepresentationBinding> {
        self.snapshot
            .binding()
            .context("player preparation has no stored representation")
    }
}
