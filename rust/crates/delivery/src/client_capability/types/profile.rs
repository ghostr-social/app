use ghostr_engine::representation::{HttpGenerationKey, HttpGenerationStamp};
use ghostr_engine::PostId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct VolatileCapabilityAuthority {
    post: PostId,
    source: String,
    generation: Option<VolatileHttpGeneration>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct VolatileHttpGeneration {
    key: HttpGenerationKey,
    epoch: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClientCapabilityProfile {
    representation: String,
    codec: Option<String>,
    dimensions: Option<(u32, u32)>,
    #[serde(default)]
    persistent: bool,
    #[serde(default)]
    volatile_authority: Option<VolatileCapabilityAuthority>,
}

impl ClientCapabilityProfile {
    pub(crate) fn try_new(
        representation: &str,
        codec: Option<&str>,
        dimensions: Option<(u32, u32)>,
    ) -> Result<Self, CapabilityProfileError> {
        let representation = required(representation)?;
        let codec = codec
            .map(required)
            .transpose()?
            .map(|value| value.to_lowercase());
        if dimensions.is_some_and(|(width, height)| width == 0 || height == 0) {
            return Err(CapabilityProfileError::ZeroDimension);
        }
        Ok(Self {
            representation,
            codec,
            dimensions,
            persistent: false,
            volatile_authority: None,
        })
    }

    pub(crate) fn with_persistent_identity(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        if persistent {
            self.volatile_authority = None;
        }
        self
    }

    pub(crate) fn with_volatile_authority(
        mut self,
        post: &PostId,
        source: &str,
        generation: Option<HttpGenerationStamp>,
    ) -> Self {
        if !self.persistent {
            self.volatile_authority = Some(VolatileCapabilityAuthority {
                post: post.clone(),
                source: source.to_owned(),
                generation: generation.map(VolatileHttpGeneration::from),
            });
        }
        self
    }

    pub(crate) fn codec(&self) -> Option<&str> {
        self.codec.as_deref()
    }

    pub(crate) const fn dimensions(&self) -> Option<(u32, u32)> {
        self.dimensions
    }

    pub(in crate::client_capability) fn applies_to(&self, requested: &Self) -> bool {
        if self.representation != requested.representation
            || self.persistent != requested.persistent
        {
            return false;
        }
        if self.persistent {
            return true;
        }
        match (&self.volatile_authority, &requested.volatile_authority) {
            (Some(recorded), Some(current)) if same_scope(recorded, current) => current
                .generation
                .as_ref()
                .is_none_or(|value| recorded.generation.as_ref() == Some(value)),
            (None, None) => {
                self.codec == requested.codec && self.dimensions == requested.dimensions
            }
            _ => false,
        }
    }

    pub(in crate::client_capability) fn is_superseded_by(&self, current: &Self) -> bool {
        !self.persistent
            && !current.persistent
            && self.representation == current.representation
            && matching_authority(
                self.volatile_authority.as_ref(),
                current.volatile_authority.as_ref(),
            )
    }

    pub(in crate::client_capability) fn is_valid(&self) -> bool {
        !self.representation.trim().is_empty()
            && self
                .codec
                .as_ref()
                .is_none_or(|value| !value.trim().is_empty())
            && self
                .dimensions
                .is_none_or(|(width, height)| width > 0 && height > 0)
            && (!self.persistent || self.volatile_authority.is_none())
    }

    pub(in crate::client_capability) const fn is_persistent(&self) -> bool {
        self.persistent
    }

    pub(in crate::client_capability) fn promote_persistence(&mut self, persistent: bool) {
        if persistent {
            self.persistent = true;
            self.volatile_authority = None;
        }
    }
}

impl From<HttpGenerationStamp> for VolatileHttpGeneration {
    fn from(stamp: HttpGenerationStamp) -> Self {
        Self {
            key: stamp.key().clone(),
            epoch: stamp.authority().epoch().value(),
        }
    }
}

fn matching_authority(
    previous: Option<&VolatileCapabilityAuthority>,
    current: Option<&VolatileCapabilityAuthority>,
) -> bool {
    matches!((previous, current), (Some(previous), Some(current))
        if same_scope(previous, current) && current.generation.is_some()
            && previous.generation != current.generation)
}

fn same_scope(left: &VolatileCapabilityAuthority, right: &VolatileCapabilityAuthority) -> bool {
    left.post == right.post && left.source == right.source
}

fn required(value: &str) -> Result<String, CapabilityProfileError> {
    let value = value.trim();
    if value.is_empty() {
        Err(CapabilityProfileError::EmptyValue)
    } else {
        Ok(value.to_owned())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityProfileError {
    EmptyValue,
    ZeroDimension,
}
