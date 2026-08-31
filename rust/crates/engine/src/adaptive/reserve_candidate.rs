use super::{CandidateSnapshot, FeedOffset, HlsCandidateSnapshot, ReserveCandidateKind};
use crate::PostId;

#[derive(Clone, Copy)]
pub(super) enum ReserveCandidate<'a> {
    Progressive(&'a CandidateSnapshot),
    Hls(&'a HlsCandidateSnapshot),
}

impl<'a> ReserveCandidate<'a> {
    pub(super) const fn progressive(self) -> Option<&'a CandidateSnapshot> {
        match self {
            Self::Progressive(candidate) => Some(candidate),
            Self::Hls(_) => None,
        }
    }

    pub(super) fn post(self) -> &'a PostId {
        match self {
            Self::Progressive(candidate) => &candidate.post,
            Self::Hls(candidate) => &candidate.post,
        }
    }

    pub(super) const fn offset(self) -> FeedOffset {
        match self {
            Self::Progressive(candidate) => candidate.feed_offset,
            Self::Hls(candidate) => candidate.feed_offset,
        }
    }

    pub(super) const fn kind(self) -> ReserveCandidateKind {
        match self {
            Self::Progressive(_) => ReserveCandidateKind::Progressive,
            Self::Hls(_) => ReserveCandidateKind::Hls,
        }
    }
}
