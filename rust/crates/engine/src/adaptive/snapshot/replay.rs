use super::PlayabilitySnapshot;

impl PlayabilitySnapshot {
    pub(crate) fn replay_sources(&self) -> Vec<String> {
        let mut sources: Vec<_> = self
            .candidates
            .iter()
            .flat_map(|candidate| {
                candidate
                    .preferred_source
                    .iter()
                    .chain(candidate.in_flight.iter().map(|action| &action.source))
                    .chain(candidate.origins.iter().map(|origin| &origin.source))
            })
            .cloned()
            .collect();
        sources.extend(
            self.hls_candidates
                .iter()
                .filter_map(|candidate| candidate.source().map(str::to_owned)),
        );
        sources
    }
}
