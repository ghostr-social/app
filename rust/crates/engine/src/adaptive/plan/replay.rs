use super::{AllocationPlan, NextReserveEvidence};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const ENTRY_LIMIT: usize = 64;

impl AllocationPlan {
    pub(crate) fn replay_sources(&self) -> Vec<String> {
        self.allocations
            .iter()
            .map(|item| item.source.clone())
            .chain(self.retained.iter().map(|item| item.source.clone()))
            .collect()
    }

    pub(crate) fn replay_project(
        &self,
        post: &impl Fn(&str) -> String,
        source: &impl Fn(&str) -> String,
    ) -> Self {
        let mut projected = self.clone();
        project_allocations(&mut projected, post, source);
        project_reserve(&mut projected, post);
        projected
    }

    pub(crate) fn replay_bounded(&self) -> bool {
        self.allocations.len() <= ENTRY_LIMIT
            && self.retained.len() <= ENTRY_LIMIT
            && self.evictions.len() <= ENTRY_LIMIT
            && self.ready_reserve.candidates.len() <= ENTRY_LIMIT
            && reserve_ranges_bounded(self)
    }
}

fn project_allocations(
    plan: &mut AllocationPlan,
    post: &impl Fn(&str) -> String,
    source: &impl Fn(&str) -> String,
) {
    for allocation in &mut plan.allocations {
        allocation.post = crate::PostId::new(post(allocation.post.as_str()));
        allocation.source = source(&allocation.source);
    }
    for retained in &mut plan.retained {
        retained.post = crate::PostId::new(post(retained.post.as_str()));
        retained.source = source(&retained.source);
    }
    for eviction in &mut plan.evictions {
        eviction.post = crate::PostId::new(post(eviction.post.as_str()));
    }
}

fn project_reserve(plan: &mut AllocationPlan, post: &impl Fn(&str) -> String) {
    for candidate in &mut plan.ready_reserve.candidates {
        candidate.post = crate::PostId::new(post(candidate.post.as_str()));
    }
    match &mut plan.next_reserve {
        NextReserveEvidence::NotApplicable => {}
        NextReserveEvidence::Ready { post: id, .. }
        | NextReserveEvidence::Structural { post: id, .. }
        | NextReserveEvidence::InFlight { post: id }
        | NextReserveEvidence::Granted { post: id, .. }
        | NextReserveEvidence::Infeasible { post: id, .. } => {
            *id = crate::PostId::new(post(id.as_str()));
        }
    }
}

fn reserve_ranges_bounded(plan: &AllocationPlan) -> bool {
    let candidate_ranges = plan.ready_reserve.candidates.iter().all(|candidate| {
        use super::ReserveCandidateState::{Planned, Preparing, Ready, Structural};
        match &candidate.state {
            Ready { startup } | Structural { startup } => startup.ranges().len() <= ENTRY_LIMIT,
            Preparing { ranges } | Planned { ranges } => ranges.len() <= ENTRY_LIMIT,
            _ => true,
        }
    });
    candidate_ranges && next_ranges_bounded(&plan.next_reserve)
}

fn next_ranges_bounded(value: &NextReserveEvidence) -> bool {
    match value {
        NextReserveEvidence::Ready { startup, .. }
        | NextReserveEvidence::Structural { startup, .. } => startup.ranges().len() <= ENTRY_LIMIT,
        _ => true,
    }
}

#[derive(Deserialize, Serialize)]
struct StartupSerde {
    ranges: Vec<crate::ByteRange>,
    playable_ms: u64,
    provenance: u8,
}

impl Serialize for StartupFootprint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        StartupSerde::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StartupFootprint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = StartupSerde::deserialize(deserializer)?;
        value
            .restore()
            .ok_or_else(|| serde::de::Error::custom("invalid startup footprint"))
    }
}

impl From<&StartupFootprint> for StartupSerde {
    fn from(value: &StartupFootprint) -> Self {
        Self {
            ranges: value.ranges().to_vec(),
            playable_ms: value.playable_ms(),
            provenance: match value.provenance() {
                StartupProvenance::WholeObjectV1 => 0,
                StartupProvenance::ClassicMp4V1 => 1,
            },
        }
    }
}

impl StartupSerde {
    fn restore(self) -> Option<StartupFootprint> {
        let provenance = match self.provenance {
            0 => StartupProvenance::WholeObjectV1,
            1 => StartupProvenance::ClassicMp4V1,
            _ => return None,
        };
        StartupFootprint::new(self.ranges, self.playable_ms, provenance)
    }
}
