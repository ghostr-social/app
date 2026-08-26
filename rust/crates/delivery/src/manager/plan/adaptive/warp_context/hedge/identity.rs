use super::super::active::ActiveContextInput;
use ghostr_engine::adaptive::{CandidateSnapshot, IdentityProof, RetrievalRequest};

pub(super) fn proof(
    evidence: &ActiveContextInput<'_>,
    candidate: &CandidateSnapshot,
    alternate: &str,
) -> Option<IdentityProof> {
    evidence
        .state
        .catalog()
        .transfer_identity(&candidate.post, alternate)?;
    match evidence.active.request() {
        RetrievalRequest::FetchRange { .. } => verified_hash(evidence, candidate, alternate),
        RetrievalRequest::FetchWhole { .. } => independent_whole(evidence, alternate),
    }
}

fn verified_hash(
    evidence: &ActiveContextInput<'_>,
    candidate: &CandidateSnapshot,
    alternate: &str,
) -> Option<IdentityProof> {
    let primary = evidence.active.identity().source().as_str();
    let value =
        evidence
            .state
            .catalog()
            .verified_mirror_digest(&candidate.post, primary, alternate)?;
    decode_hash(value).map(IdentityProof::VerifiedHash)
}

fn independent_whole(evidence: &ActiveContextInput<'_>, alternate: &str) -> Option<IdentityProof> {
    evidence
        .inputs
        .independent_sources
        .get(evidence.active.post())
        .is_some_and(|sources| sources.contains(alternate))
        .then_some(IdentityProof::IndependentWhole)
}

fn decode_hash(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut result = [0; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        result[index] = nibble(pair[0])?
            .checked_mul(16)?
            .checked_add(nibble(pair[1])?)?;
    }
    Some(result)
}

fn nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
