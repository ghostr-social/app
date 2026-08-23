use ghostr_engine::adaptive::DecisionOutcome;

pub(super) fn is(error: &anyhow::Error) -> bool {
    crate::chunk::whole_body_policy::is(error)
}

pub(super) fn decision(error: &anyhow::Error) -> DecisionOutcome {
    match crate::chunk::whole_body_bound::from_error(error) {
        Some(_) => DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 0,
        },
        None => DecisionOutcome::Failed {
            class: "warp_whole_body_limit".into(),
            elapsed_ms: 0,
        },
    }
}

pub(super) fn stored_bytes(outcome: &anyhow::Result<crate::chunk::downloader::ChunkResult>) -> u64 {
    outcome.as_ref().map_or(0, |result| result.bytes_written)
}
