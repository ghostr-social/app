use core::num::NonZeroU64;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RenditionId(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenditionError {
    EmptyId,
    ZeroBitrate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rendition {
    id: RenditionId,
    bitrate: NonZeroU64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenditionSetError {
    Empty,
    DuplicateId,
    DuplicateBitrate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenditionSet {
    renditions: Vec<Rendition>,
}

impl RenditionId {
    /// # Errors
    ///
    /// Returns [`RenditionError::EmptyId`] when the identifier is blank.
    pub(crate) fn try_new(value: impl Into<String>) -> Result<Self, RenditionError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(RenditionError::EmptyId);
        }
        Ok(Self(value))
    }
}

impl Rendition {
    /// # Errors
    ///
    /// Returns an error when the identifier is blank or the bitrate is zero.
    pub(crate) fn try_new(
        id: impl Into<String>,
        bitrate_bits_per_second: u64,
    ) -> Result<Self, RenditionError> {
        Ok(Self {
            id: RenditionId::try_new(id)?,
            bitrate: NonZeroU64::new(bitrate_bits_per_second).ok_or(RenditionError::ZeroBitrate)?,
        })
    }

    pub(crate) fn id(&self) -> &RenditionId {
        &self.id
    }

    pub(crate) fn bitrate_bits_per_second(&self) -> u64 {
        self.bitrate.get()
    }
}

impl RenditionSet {
    /// # Errors
    ///
    /// Returns an error for an empty set or duplicate identifiers or bitrates.
    pub(crate) fn try_new(mut renditions: Vec<Rendition>) -> Result<Self, RenditionSetError> {
        if renditions.is_empty() {
            return Err(RenditionSetError::Empty);
        }
        renditions.sort_by(|left, right| left.id.cmp(&right.id));
        if renditions.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(RenditionSetError::DuplicateId);
        }
        renditions.sort_by(rendition_order);
        if renditions.windows(2).any(same_bitrate) {
            return Err(RenditionSetError::DuplicateBitrate);
        }
        Ok(Self { renditions })
    }

    pub(super) fn find(&self, id: Option<&RenditionId>) -> Option<&Rendition> {
        let id = id?;
        self.renditions
            .iter()
            .find(|rendition| rendition.id() == id)
    }

    pub(super) fn highest_at_or_below(&self, limit: u64) -> &Rendition {
        self.renditions
            .iter()
            .rev()
            .find(|rendition| rendition.bitrate_bits_per_second() <= limit)
            .unwrap_or(&self.renditions[0])
    }
}

fn rendition_order(left: &Rendition, right: &Rendition) -> core::cmp::Ordering {
    left.bitrate
        .cmp(&right.bitrate)
        .then_with(|| left.id.cmp(&right.id))
}

fn same_bitrate(pair: &[Rendition]) -> bool {
    pair[0].bitrate == pair[1].bitrate
}
