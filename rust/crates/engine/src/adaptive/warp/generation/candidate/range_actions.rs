use super::super::allocation::AllocationSpec;
use super::super::builder::Builder;
use crate::adaptive::{ActionKind, CandidateSnapshot};

impl Builder<'_> {
    pub(super) fn add_range_actions(&mut self, candidate: &CandidateSnapshot) {
        self.add_prefix(candidate);
        let prefix = self.action_id(candidate, |kind| matches!(kind, ActionKind::Prefix(_)));
        self.add_tail(candidate, prefix);
        self.add_continuation(candidate);
        self.add_cache_upgrade(candidate);
    }

    fn add_prefix(&mut self, candidate: &CandidateSnapshot) {
        let Some(source) = self.request_source(candidate) else {
            return;
        };
        if !candidate.needs_bootstrap() {
            return;
        }
        let Some(missing) = crate::adaptive::ranges::missing(candidate)
            .into_iter()
            .find(|item| item.bytes.start < 65_536)
        else {
            return;
        };
        let range = bounded_range(missing.bytes, 65_536);
        let kind = ActionKind::Prefix(range);
        if self.contains(candidate, &kind) {
            return;
        }
        let allocation = self.allocation(candidate, AllocationSpec::range(range, source, 0));
        self.push_transfer(candidate, kind, allocation, &[]);
    }

    fn add_tail(&mut self, candidate: &CandidateSnapshot, prefix: Option<u16>) {
        let (Some(probe), Some(source)) =
            (candidate.timeline_probe, self.request_source(candidate))
        else {
            return;
        };
        let kind = ActionKind::Tail(probe.bytes);
        if self.contains(candidate, &kind) {
            return;
        }
        let allocation = self.allocation(candidate, AllocationSpec::range(probe.bytes, source, 0));
        let dependencies: Vec<_> = prefix.into_iter().collect();
        self.push_transfer(candidate, kind, allocation, &dependencies);
    }

    fn add_continuation(&mut self, candidate: &CandidateSnapshot) {
        let Some(source) = self.request_source(candidate) else {
            return;
        };
        let Some(playable) = crate::adaptive::ranges::missing(candidate)
            .into_iter()
            .next()
        else {
            return;
        };
        let range = bounded_range(playable.bytes, self.snapshot.request_slice_bytes);
        let kind = ActionKind::FetchRange(range);
        if self.contains(candidate, &kind) {
            return;
        }
        let allocation = self.allocation(
            candidate,
            AllocationSpec::range(range, source, playable.playable_ms),
        );
        self.push_transfer(candidate, kind, allocation, &[]);
    }

    fn add_cache_upgrade(&mut self, candidate: &CandidateSnapshot) {
        if candidate.present.is_empty() {
            return;
        }
        let (Some(source), Some(missing)) = (
            self.request_source(candidate),
            crate::adaptive::ranges::missing(candidate)
                .into_iter()
                .next(),
        ) else {
            return;
        };
        let range = bounded_range(missing.bytes, self.snapshot.request_slice_bytes);
        let kind = ActionKind::CacheUpgrade(range);
        let allocation = self.allocation(candidate, AllocationSpec::cache(range, source));
        self.push_transfer(candidate, kind, allocation, &[]);
    }
}

fn bounded_range(range: crate::ByteRange, maximum: u64) -> crate::ByteRange {
    crate::ByteRange::new(
        range.start,
        range.start.saturating_add(maximum).min(range.end),
    )
}
