use super::{identity, RepresentationBinding, RepresentationId};
use crate::adaptive::TransformKind;
use crate::VideoMeta;

impl RepresentationBinding {
    pub fn source_representation(&self) -> &RepresentationId {
        self.derived_from.as_ref().unwrap_or(&self.representation)
    }

    pub fn matches_or_derives_from(&self, meta: &VideoMeta) -> bool {
        let representation = RepresentationId::from_meta(meta);
        self.representation == representation || self.derived_from.as_ref() == Some(&representation)
    }

    pub fn derive_transform(&self, kind: TransformKind, digest: &str) -> Option<Self> {
        let representation = identity::transformed(&self.representation.0, kind, digest)?;
        Some(Self {
            post: self.post.clone(),
            representation: RepresentationId(representation),
            derived_from: Some(self.representation.clone()),
            generation: self.generation,
            sources: Vec::new(),
        })
    }

    pub fn derives_from(&self, input: &Self) -> bool {
        self.post == input.post
            && self.generation == input.generation
            && self.derived_from.as_ref() == Some(&input.representation)
    }
}
