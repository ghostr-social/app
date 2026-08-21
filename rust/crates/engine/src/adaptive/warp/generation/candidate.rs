use super::allocation::{classify, resources, source, AllocationSpec};
use super::builder::{Builder, NodeInput};
use super::prediction::transform_prediction;
use super::{GeneratedAction, PlannerCommand};
use crate::adaptive::{ActionKind, CandidateSnapshot};

impl Builder<'_> {
    pub(super) fn add_candidate(&mut self, candidate: &CandidateSnapshot) {
        self.add_head(candidate);
        self.add_base(candidate);
        self.add_prefix(candidate);
        let prefix = self.action_id(candidate, |kind| matches!(kind, ActionKind::Prefix(_)));
        self.add_tail(candidate, prefix);
        self.add_continuation(candidate);
        let whole = self.add_whole(candidate);
        self.add_cache_upgrade(candidate);
        self.add_transform(candidate, whole);
        self.add_active(candidate);
    }

    fn add_head(&mut self, candidate: &CandidateSnapshot) {
        let Some(source) = source(candidate) else {
            return;
        };
        let head_suppressed = self
            .context
            .candidate(&candidate.post)
            .is_some_and(|item| item.head_probe != super::super::HeadProbeHistory::Unobserved);
        if !candidate.needs_bootstrap() || head_suppressed {
            return;
        }
        let kind = ActionKind::Head;
        let prediction = self.prediction(candidate, &kind, source);
        let input = NodeInput::new(kind.clone(), source, prediction, &[]);
        let mut node = self.node(candidate, input);
        node.resources = resources(&kind);
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::ProbeHead {
                post: candidate.post.clone(),
                source: source.to_owned(),
                authority: super::allocation::authority(candidate, self.snapshot, self.base.mode),
            },
        });
    }

    fn add_base(&mut self, candidate: &CandidateSnapshot) {
        let allocations: Vec<_> = self
            .base
            .allocations
            .iter()
            .filter(|item| item.post == candidate.post)
            .cloned()
            .collect();
        for allocation in allocations {
            let allocation = self.normalize_playability(candidate, allocation);
            self.push_transfer(candidate, classify(&allocation), allocation, &[]);
        }
    }

    fn add_prefix(&mut self, candidate: &CandidateSnapshot) {
        let Some(source) = source(candidate) else {
            return;
        };
        if !candidate.needs_bootstrap() {
            return;
        }
        let Some(missing) = super::super::super::ranges::missing(candidate)
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
        let (Some(probe), Some(source)) = (candidate.timeline_probe, source(candidate)) else {
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
        let Some(source) = source(candidate) else {
            return;
        };
        let Some(playable) = super::super::super::ranges::missing(candidate)
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

    fn add_whole(&mut self, candidate: &CandidateSnapshot) -> Option<u16> {
        let (Some(total), Some(source)) = (candidate.total_bytes, source(candidate)) else {
            return None;
        };
        if candidate.finalized {
            return None;
        }
        let kind = ActionKind::FetchWhole {
            maximum_bytes: total,
        };
        if self.contains(candidate, &kind) {
            return self.action_id(candidate, |item| {
                matches!(item, ActionKind::FetchWhole { .. })
            });
        }
        let allocation = self.allocation(
            candidate,
            AllocationSpec::whole(total, source, candidate.duration_ms),
        );
        Some(self.push_transfer(candidate, kind, allocation, &[]))
    }

    fn add_cache_upgrade(&mut self, candidate: &CandidateSnapshot) {
        if candidate.present.is_empty() {
            return;
        }
        let (Some(source), Some(missing)) = (
            source(candidate),
            super::super::super::ranges::missing(candidate)
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

    fn add_transform(&mut self, candidate: &CandidateSnapshot, whole: Option<u16>) {
        let Some(transform) = self
            .context
            .candidate(&candidate.post)
            .and_then(|item| item.capability.required_transform())
        else {
            return;
        };
        let kind = ActionKind::Transform(transform.kind);
        let requires: Vec<_> = whole.into_iter().collect();
        let prediction = transform_prediction(candidate, transform.estimated_cpu_ms);
        let input = NodeInput::new(kind, "local-transform", prediction, &requires);
        let mut node = self.node(candidate, input);
        node.resources = super::super::ResourceCost::new(
            0,
            transform.output_upper_bytes,
            transform.estimated_cpu_ms,
            0,
        );
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::Transform {
                post: candidate.post.clone(),
                kind: transform.kind,
            },
        });
    }
}

fn bounded_range(range: crate::ByteRange, maximum: u64) -> crate::ByteRange {
    crate::ByteRange::new(
        range.start,
        range.start.saturating_add(maximum).min(range.end),
    )
}
