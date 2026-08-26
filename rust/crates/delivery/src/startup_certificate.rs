//! Exact store authority for one structurally startable media closure.

use ghostr_engine::media_timeline::StartupFootprint;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::{
    ContentRevision, StoredEvidenceId, StoredMediaSnapshot,
};

const CERTIFICATE_PROFILE: u16 = 1;

#[derive(Clone, Eq, PartialEq)]
pub struct StartupCertificate {
    binding: RepresentationBinding,
    revision: ContentRevision,
    total: u64,
    startup: StartupFootprint,
    stored: StoredEvidenceId,
    profile: u16,
}

impl core::fmt::Debug for StartupCertificate {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("StartupCertificate")
            .field("post", self.binding.post())
            .field("revision", &self.revision)
            .field("total", &self.total)
            .field("ranges", &self.startup.ranges())
            .field("provenance", &self.startup.provenance())
            .field("profile", &self.profile)
            .finish_non_exhaustive()
    }
}

impl StartupCertificate {
    pub fn issue(startup: StartupFootprint, snapshot: &StoredMediaSnapshot) -> Option<Self> {
        let binding = snapshot.binding()?.clone();
        let total = snapshot.total_len()?;
        if startup.ranges().iter().any(|range| range.end > total) {
            return None;
        }
        let spans: Vec<_> = startup
            .ranges()
            .iter()
            .map(|range| range.start..range.end)
            .collect();
        let stored = snapshot.evidence_id_for(&spans)?;
        Some(Self {
            binding,
            revision: snapshot.revision(),
            total,
            startup,
            stored,
            profile: CERTIFICATE_PROFILE,
        })
    }

    fn post(&self) -> &PostId {
        self.binding.post()
    }

    fn startup(&self) -> &StartupFootprint {
        &self.startup
    }

    pub fn matches(&self, post: &PostId, startup: &StartupFootprint) -> bool {
        self.post() == post && self.startup() == startup
    }

    pub fn still_valid_in(&self, snapshot: &StoredMediaSnapshot) -> bool {
        let spans: Vec<_> = self
            .startup
            .ranges()
            .iter()
            .map(|range| range.start..range.end)
            .collect();
        snapshot.binding() == Some(&self.binding)
            && snapshot.revision() == self.revision
            && snapshot.total_len() == Some(self.total)
            && self.profile == CERTIFICATE_PROFILE
            && snapshot.evidence_id_for(&spans) == Some(self.stored)
    }
}
