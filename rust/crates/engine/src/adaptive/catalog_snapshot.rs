use super::{CandidateSnapshot, FeedOffset, InFlightAction, OriginHealth, ViewProbability};
use crate::catalog::Catalog;
use crate::{ByteRange, EngineParams, PostId};
use std::collections::HashSet;

mod playable;
mod resolved;
mod startup;

pub struct CandidateEvidence {
    pub post: PostId,
    pub feed_offset: FeedOffset,
    pub view_probability: ViewProbability,
    pub present: Vec<ByteRange>,
    pub stored_total: Option<u64>,
    pub continuation_source: Option<String>,
    pub independent_object_sources: HashSet<String>,
    pub recently_evicted: Vec<ByteRange>,
    pub in_flight: Vec<InFlightAction>,
    pub origins: Vec<OriginHealth>,
}

pub fn candidate_snapshot(
    catalog: &Catalog,
    params: &EngineParams,
    evidence: CandidateEvidence,
) -> Option<CandidateSnapshot> {
    candidate_snapshot_at(catalog, params, evidence, 0)
}

pub fn candidate_snapshot_at(
    catalog: &Catalog,
    params: &EngineParams,
    evidence: CandidateEvidence,
    observed_at_ms: u64,
) -> Option<CandidateSnapshot> {
    let resolved = resolved::resolve(catalog, params, &evidence, observed_at_ms)?;
    Some(resolved.into_snapshot(evidence))
}
