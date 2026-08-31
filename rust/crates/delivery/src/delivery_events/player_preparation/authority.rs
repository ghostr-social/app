use crate::segmented::HlsPreparedAssetAuthority;
use core::fmt::{Debug, Formatter};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use sha2::{Digest as _, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreparationAuthority {
    pub(super) post: PostId,
    media: PlayerPreparationMediaAuthority,
    asset: PlayerPreparationAssetFingerprint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PlayerPreparationMediaAuthority {
    Progressive {
        binding: RepresentationBinding,
        revision: ContentRevision,
    },
    Hls(HlsPreparedAssetAuthority),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerPreparationClaim {
    post: PostId,
    representation: String,
    asset: PlayerPreparationAssetFingerprint,
}

impl PlayerPreparationAuthority {
    pub fn try_new(
        post: PostId,
        binding: RepresentationBinding,
        revision: ContentRevision,
        asset_id: impl AsRef<str>,
    ) -> Option<Self> {
        let asset = PlayerPreparationAssetFingerprint::capture(asset_id.as_ref())?;
        (binding.post() == &post).then_some(Self {
            post,
            media: PlayerPreparationMediaAuthority::Progressive { binding, revision },
            asset,
        })
    }

    pub fn try_new_hls(
        authority: HlsPreparedAssetAuthority,
        asset_id: impl AsRef<str>,
    ) -> Option<Self> {
        let asset = PlayerPreparationAssetFingerprint::capture(asset_id.as_ref())?;
        Some(Self {
            post: authority.post().clone(),
            media: PlayerPreparationMediaAuthority::Hls(authority),
            asset,
        })
    }

    pub(super) fn progressive_binding(&self) -> Option<&RepresentationBinding> {
        match &self.media {
            PlayerPreparationMediaAuthority::Progressive { binding, .. } => Some(binding),
            PlayerPreparationMediaAuthority::Hls(_) => None,
        }
    }

    pub(super) fn progressive_revision(&self) -> Option<ContentRevision> {
        match &self.media {
            PlayerPreparationMediaAuthority::Progressive { revision, .. } => Some(*revision),
            PlayerPreparationMediaAuthority::Hls(_) => None,
        }
    }

    pub(super) fn hls_authority(&self) -> Option<&HlsPreparedAssetAuthority> {
        match &self.media {
            PlayerPreparationMediaAuthority::Hls(authority) => Some(authority),
            PlayerPreparationMediaAuthority::Progressive { .. } => None,
        }
    }

    pub(super) fn accepts(&self, claim: &PlayerPreparationClaim) -> bool {
        self.post == claim.post
            && self.representation_fingerprint() == claim.representation
            && self.asset == claim.asset
    }

    fn representation_fingerprint(&self) -> &str {
        match &self.media {
            PlayerPreparationMediaAuthority::Progressive { binding, .. } => {
                binding.representation().fingerprint()
            }
            PlayerPreparationMediaAuthority::Hls(authority) => {
                authority.representation_id().fingerprint()
            }
        }
    }
}

impl PlayerPreparationClaim {
    pub fn try_new(
        post: PostId,
        representation: impl Into<String>,
        asset_id: impl AsRef<str>,
    ) -> Option<Self> {
        let representation = representation.into();
        let valid_representation = representation.len() == 64
            && representation.bytes().all(|byte| byte.is_ascii_hexdigit());
        let asset = PlayerPreparationAssetFingerprint::capture(asset_id.as_ref())?;
        valid_representation.then_some(Self {
            post,
            representation,
            asset,
        })
    }

    pub fn post(&self) -> &PostId {
        &self.post
    }

    pub(super) fn from_authority(authority: &PlayerPreparationAuthority) -> Self {
        Self {
            post: authority.post.clone(),
            representation: authority.representation_fingerprint().to_owned(),
            asset: authority.asset.clone(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct PlayerPreparationAssetFingerprint([u8; 32]);

impl PlayerPreparationAssetFingerprint {
    fn capture(raw: &str) -> Option<Self> {
        if raw.is_empty() {
            return None;
        }
        Some(Self(Sha256::digest(raw.as_bytes()).into()))
    }
}

impl Debug for PlayerPreparationAssetFingerprint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("[redacted]")
    }
}
